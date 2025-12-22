use chrono::{DateTime, Datelike, Local};
use rustc_hash::FxHashMap;
use std::fs;
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use typst::diag::{FileError, FileResult, StrResult};
use typst::foundations::{Bytes, Datetime, Dict};
use typst::syntax::{FileId, Lines, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World};
use typst_kit::{
    fonts::{FontSearcher, Fonts},
    package::PackageStorage,
};

use crate::{Input, download::SlientDownload};

/// A world that provides access to the operating system.
pub struct SystemWorld {
    /// The working directory.
    workdir: Option<PathBuf>,
    /// The root relative to which absolute paths are resolved.
    root: PathBuf,
    /// The input path.
    main: FileId,
    /// Reusable file id for in-memory (bytes) inputs.
    bytes_main: Option<FileId>,
    /// Typst's standard library.
    library: LazyHash<Library>,
    /// Metadata about discovered fonts.
    book: LazyHash<FontBook>,
    /// Locations of and storage for lazily loaded fonts.
    fonts: Arc<typst_kit::fonts::Fonts>,
    /// Maps file ids to source files and buffers.
    slots: Mutex<FxHashMap<FileId, FileSlot>>,
    /// Holds information about where packages are stored.
    package_storage: PackageStorage,
    /// The current datetime if requested. This is stored here to ensure it is
    /// always the same within one compilation. Reset between compilations.
    now: OnceLock<DateTime<Local>>,
}

impl World for SystemWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.slot(id, |slot| slot.source(&self.root, &self.package_storage))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.slot(id, |slot| slot.file(&self.root, &self.package_storage))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.fonts[index].get()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let now = self.now.get_or_init(chrono::Local::now);

        let naive = match offset {
            None => now.naive_local(),
            Some(o) => now.naive_utc() + chrono::Duration::hours(o),
        };

        Datetime::from_ymd(
            naive.year(),
            naive.month().try_into().ok()?,
            naive.day().try_into().ok()?,
        )
    }
}

impl SystemWorld {
    pub fn builder(root: PathBuf, input: Input) -> SystemWorldBuilder {
        SystemWorldBuilder::new(root, input)
    }

    /// Update the primary input for the world, reusing the existing
    /// [`SystemWorld`] configuration (fonts, packages, etc.).
    pub fn set_input(&mut self, input: Input) -> StrResult<()> {
        self.reset();
        match input {
            Input::Path(path) => self.configure_path_input(path),
            Input::Bytes(bytes) => self.configure_bytes_input(bytes),
        }
    }

    /// Reset the compilation state in preparation of a new compilation.
    pub fn reset(&mut self) {
        let mut slots = self.slots.lock().unwrap();
        for slot in slots.values_mut() {
            slot.reset();
        }
        // Reset the datetime for each compilation
        self.now.take();
    }

    /// Update the sys_inputs for the world.
    pub fn set_inputs(&mut self, inputs: Dict) {
        self.library = LazyHash::new(
            Library::builder()
                .with_features(vec![typst::Feature::Html].into_iter().collect())
                .with_inputs(inputs)
                .build(),
        );
    }

    /// Access the canonical slot for the given file id.
    fn slot<F, T>(&self, id: FileId, f: F) -> T
    where
        F: FnOnce(&mut FileSlot) -> T,
    {
        let mut map = self.slots.lock().unwrap();
        f(map.entry(id).or_insert_with(|| FileSlot::new(id)))
    }

    /// The root relative to which absolute paths are resolved.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The current working directory.
    pub fn workdir(&self) -> &Path {
        self.workdir.as_deref().unwrap_or(Path::new("."))
    }

    /// Lookup a source file by id.
    #[track_caller]
    pub fn lookup(&self, id: FileId) -> Lines<String> {
        self.slot(id, |slot| {
            if let Some(source) = slot.source.get() {
                let source = source.as_ref().expect("file is not valid");
                source.lines().clone()
            } else if let Some(bytes) = slot.file.get() {
                let bytes = bytes.as_ref().expect("file is not valid");
                Lines::try_from(bytes).expect("file is not valid utf-8")
            } else {
                panic!("file id does not point to any source file");
            }
        })
    }
}

pub struct SystemWorldBuilder {
    root: PathBuf,
    input: Input,
    fonts: Option<Arc<Fonts>>,
    inputs: Dict,
    package_path: Option<PathBuf>,
}

impl SystemWorldBuilder {
    pub fn new(root: PathBuf, input: Input) -> Self {
        Self {
            root,
            input,
            fonts: None,
            inputs: Dict::default(),
            package_path: None,
        }
    }

    pub fn package_path(mut self, package_path: Option<PathBuf>) -> Self {
        self.package_path = package_path;
        self
    }

    pub fn fonts(mut self, fonts: Option<Arc<Fonts>>) -> Self {
        self.fonts = fonts;
        self
    }

    pub fn inputs(mut self, inputs: Dict) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn build(self) -> StrResult<SystemWorld> {
        let fonts = match self.fonts {
            Some(fonts) => fonts,
            None => Arc::new(FontSearcher::new().include_system_fonts(true).search()),
        };

        let mut slots = FxHashMap::default();
        let mut bytes_main = None;
        let main = match self.input {
            Input::Path(path) => {
                // Resolve the virtual path of the main file within the project root.
                let path = path
                    .canonicalize()
                    .map_err(|err| format!("Failed to canonicalize path: {}", err))?;
                FileId::new(
                    None,
                    VirtualPath::within_root(&path, &self.root)
                        .ok_or("input file must be contained in project root")?,
                )
            }
            Input::Bytes(bytes) => {
                // Fake file ID
                let file_id = FileId::new_fake(VirtualPath::new("<bytes>"));
                let mut file_slot = FileSlot::new(file_id);
                file_slot
                    .source
                    .init(Source::new(file_id, decode_utf8(&bytes)?.to_string()));
                file_slot.file.init(Bytes::new(bytes));
                slots.insert(file_id, file_slot);
                bytes_main = Some(file_id);
                file_id
            }
        };
        let world = SystemWorld {
            workdir: std::env::current_dir().ok(),
            root: self.root,
            main,
            bytes_main,
            library: LazyHash::new(
                Library::builder()
                    .with_features(vec![typst::Feature::Html].into_iter().collect())
                    .with_inputs(self.inputs)
                    .build(),
            ),
            book: LazyHash::new(fonts.book.clone()),
            fonts,
            slots: Mutex::new(slots),
            package_storage: PackageStorage::new(
                None,
                self.package_path,
                crate::download::downloader(),
            ),
            now: OnceLock::new(),
        };
        Ok(world)
    }
}

/// Holds canonical data for all paths pointing to the same entity.
///
/// Both fields can be populated if the file is both imported and read().
struct FileSlot {
    /// The slot's canonical file id.
    id: FileId,
    /// The lazily loaded and incrementally updated source file.
    source: SlotCell<Source>,
    /// The lazily loaded raw byte buffer.
    file: SlotCell<Bytes>,
}

impl FileSlot {
    /// Create a new path slot.
    fn new(id: FileId) -> Self {
        Self {
            id,
            file: SlotCell::new(),
            source: SlotCell::new(),
        }
    }

    /// Create a slot backed by explicitly provided bytes.
    fn from_inline_bytes(id: FileId, bytes: Vec<u8>) -> FileResult<Self> {
        let mut slot = FileSlot::new(id);
        slot.source
            .init(Source::new(id, decode_utf8(&bytes)?.to_string()));
        slot.file.init(Bytes::new(bytes));
        Ok(slot)
    }

    /// Marks the file as not yet accessed in preparation of the next
    /// compilation.
    fn reset(&mut self) {
        self.source.reset();
        self.file.reset();
    }

    fn source(
        &mut self,
        project_root: &Path,
        package_storage: &PackageStorage,
    ) -> FileResult<Source> {
        self.source.get_or_init(
            || read(self.id, project_root, package_storage),
            |data, prev| {
                let text = decode_utf8(&data)?;
                if let Some(mut prev) = prev {
                    prev.replace(text);
                    Ok(prev)
                } else {
                    Ok(Source::new(self.id, text.into()))
                }
            },
        )
    }

    /// Retrieve the file's bytes.
    fn file(&mut self, project_root: &Path, package_storage: &PackageStorage) -> FileResult<Bytes> {
        self.file.get_or_init(
            || read(self.id, project_root, package_storage),
            |data, _| Ok(Bytes::new(data)),
        )
    }
}

impl SystemWorld {
    fn configure_path_input(&mut self, path: PathBuf) -> StrResult<()> {
        let path = path
            .canonicalize()
            .map_err(|err| format!("Failed to canonicalize path: {}", err))?;
        let Some(vpath) = VirtualPath::within_root(&path, &self.root) else {
            return Err("input file must be contained in project root".into());
        };
        self.main = FileId::new(None, vpath);
        Ok(())
    }

    fn configure_bytes_input(&mut self, bytes: Vec<u8>) -> StrResult<()> {
        let id = if let Some(id) = self.bytes_main {
            id
        } else {
            let id = FileId::new_fake(VirtualPath::new("<bytes>"));
            self.bytes_main = Some(id);
            id
        };

        let slot = FileSlot::from_inline_bytes(id, bytes).map_err(|err| err.to_string())?;
        self.main = id;
        self.slots.lock().unwrap().insert(id, slot);
        Ok(())
    }
}
/// The path of the slot on the system.
fn system_path(root: &Path, id: FileId, package_storage: &PackageStorage) -> FileResult<PathBuf> {
    // Determine the root path relative to which the file path
    // will be resolved.
    let buf;
    let mut root = root;
    if let Some(spec) = id.package() {
        buf = package_storage.prepare_package(spec, &mut SlientDownload(&spec))?;
        root = &buf;
    }

    // Join the path to the root. If it tries to escape, deny
    // access. Note: It can still escape via symlinks.
    id.vpath().resolve(root).ok_or(FileError::AccessDenied)
}

/// Lazily processes data for a file.
struct SlotCell<T> {
    /// The processed data.
    data: Option<FileResult<T>>,
    /// A hash of the raw file contents / access error.
    fingerprint: u128,
    /// Whether the slot has been accessed in the current compilation.
    accessed: bool,
}

impl<T: Clone> SlotCell<T> {
    /// Creates a new, empty cell.
    fn new() -> Self {
        Self {
            data: None,
            fingerprint: 0,
            accessed: false,
        }
    }

    /// Marks the cell as not yet accessed in preparation of the next
    /// compilation.
    fn reset(&mut self) {
        self.accessed = false;
    }

    /// Gets the contents of the cell.
    fn get(&self) -> Option<&FileResult<T>> {
        self.data.as_ref()
    }

    fn init(&mut self, data: T) {
        self.data = Some(Ok(data));
        self.accessed = true;
    }

    /// Gets the contents of the cell or initialize them.
    fn get_or_init(
        &mut self,
        load: impl FnOnce() -> FileResult<Vec<u8>>,
        f: impl FnOnce(Vec<u8>, Option<T>) -> FileResult<T>,
    ) -> FileResult<T> {
        // If we accessed the file already in this compilation, retrieve it.
        if mem::replace(&mut self.accessed, true)
            && let Some(data) = &self.data
        {
            return data.clone();
        }

        // Read and hash the file.
        let result = load();
        let fingerprint = typst::utils::hash128(&result);

        // If the file contents didn't change, yield the old processed data.
        if mem::replace(&mut self.fingerprint, fingerprint) == fingerprint
            && let Some(data) = &self.data
        {
            return data.clone();
        }

        let prev = self.data.take().and_then(Result::ok);
        let value = result.and_then(|data| f(data, prev));
        self.data = Some(value.clone());

        value
    }
}

fn read(id: FileId, project_root: &Path, package_storage: &PackageStorage) -> FileResult<Vec<u8>> {
    read_from_disk(&system_path(project_root, id, package_storage)?)
}

fn read_from_disk(path: &Path) -> FileResult<Vec<u8>> {
    let f = |e| FileError::from_io(e, path);
    if fs::metadata(path).map_err(f)?.is_dir() {
        Err(FileError::IsDirectory)
    } else {
        fs::read(path).map_err(f)
    }
}

/// Decode UTF-8 with an optional BOM.
fn decode_utf8(buf: &[u8]) -> FileResult<&str> {
    // Remove UTF-8 BOM.
    Ok(std::str::from_utf8(
        buf.strip_prefix(b"\xef\xbb\xbf").unwrap_or(buf),
    )?)
}

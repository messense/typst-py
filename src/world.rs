use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use chrono::{DateTime, Datelike, Local};
use ecow::eco_format;
use typst::diag::{FileError, FileResult, StrResult};
use typst::foundations::{Bytes, Datetime, Dict};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryBuilder, World};
use typst_kit::{
    fonts::{FontSearcher, FontSlot},
    package::PackageStorage,
};

use std::collections::HashMap;

use crate::download::SlientDownload;

/// A world that provides access to the operating system.
pub struct SystemWorld {
    /// The working directory.
    workdir: Option<PathBuf>,
    /// The canonical path to the input file.
    input: PathBuf,
    /// The root relative to which absolute paths are resolved.
    root: PathBuf,
    /// The input path.
    main: FileId,
    /// Typst's standard library.
    library: LazyHash<Library>,
    /// Metadata about discovered fonts.
    book: LazyHash<FontBook>,
    /// Locations of and storage for lazily loaded fonts.
    fonts: Vec<FontSlot>,
    /// Maps file ids to source files and buffers.
    slots: Mutex<HashMap<FileId, FileSlot>>,
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
        self.fonts[index].get()
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
    pub fn builder(root: PathBuf, main: PathBuf) -> SystemWorldBuilder {
        SystemWorldBuilder::new(root, main)
    }

    /// Access the canonical slot for the given file id.
    fn slot<F, T>(&self, id: FileId, f: F) -> T
    where
        F: FnOnce(&mut FileSlot) -> T,
    {
        let mut map = self.slots.lock().unwrap();
        f(map.entry(id).or_insert_with(|| FileSlot::new(id)))
    }

    /// The id of the main source file.
    pub fn main(&self) -> FileId {
        self.main
    }

    /// The root relative to which absolute paths are resolved.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The current working directory.
    pub fn workdir(&self) -> &Path {
        self.workdir.as_deref().unwrap_or(Path::new("."))
    }

    /// Reset the compilation state in preparation of a new compilation.
    pub fn reset(&mut self) {
        for slot in self.slots.lock().unwrap().values_mut() {
            slot.reset();
        }
        self.now.take();
    }

    /// Return the canonical path to the input file.
    pub fn input(&self) -> &PathBuf {
        &self.input
    }

    /// Lookup a source file by id.
    #[track_caller]
    pub fn lookup(&self, id: FileId) -> Source {
        self.source(id)
            .expect("file id does not point to any source file")
    }
}

pub struct SystemWorldBuilder {
    root: PathBuf,
    main: PathBuf,
    font_paths: Vec<PathBuf>,
    inputs: Dict,
}

impl SystemWorldBuilder {
    pub fn new(root: PathBuf, main: PathBuf) -> Self {
        Self {
            root,
            main,
            font_paths: Vec::new(),
            inputs: Dict::default(),
        }
    }

    pub fn font_paths(mut self, font_paths: Vec<PathBuf>) -> Self {
        self.font_paths = font_paths;
        self
    }

    pub fn inputs(mut self, inputs: Dict) -> Self {
        self.inputs = inputs;
        self
    }

    pub fn build(self) -> StrResult<SystemWorld> {
        let fonts = FontSearcher::new().search_with(&self.font_paths);

        let input = self.main.canonicalize().map_err(|_| {
            eco_format!("input file not found (searched at {})", self.main.display())
        })?;
        // Resolve the virtual path of the main file within the project root.
        let main_path = VirtualPath::within_root(&self.main, &self.root)
            .ok_or("input file must be contained in project root")?;

        let world = SystemWorld {
            workdir: std::env::current_dir().ok(),
            input,
            root: self.root,
            main: FileId::new(None, main_path),
            library: LazyHash::new(LibraryBuilder::default().with_inputs(self.inputs).build()),
            book: LazyHash::new(fonts.book),
            fonts: fonts.fonts,
            slots: Mutex::default(),
            package_storage: PackageStorage::new(None, None, crate::download::downloader()),
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

    /// Marks the file as not yet accessed in preparation of the next
    /// compilation.
    fn reset(&mut self) {
        self.source.reset();
        self.file.reset();
    }

    fn source(&mut self, root: &Path, package_storage: &PackageStorage) -> FileResult<Source> {
        let id = self.id;
        self.source.get_or_init(
            || system_path(root, id, package_storage),
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

    fn file(&mut self, root: &Path, package_storage: &PackageStorage) -> FileResult<Bytes> {
        let id = self.id;
        self.file.get_or_init(
            || system_path(root, id, package_storage),
            |data, _| Ok(data.into()),
        )
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

    /// Gets the contents of the cell or initialize them.
    fn get_or_init(
        &mut self,
        path: impl FnOnce() -> FileResult<PathBuf>,
        f: impl FnOnce(Vec<u8>, Option<T>) -> FileResult<T>,
    ) -> FileResult<T> {
        // If we accessed the file already in this compilation, retrieve it.
        if std::mem::replace(&mut self.accessed, true) {
            if let Some(data) = &self.data {
                return data.clone();
            }
        }

        // Read and hash the file.
        let result = path().and_then(|p| read(&p));
        let fingerprint = typst::utils::hash128(&result);

        // If the file contents didn't change, yield the old processed data.
        if std::mem::replace(&mut self.fingerprint, fingerprint) == fingerprint {
            if let Some(data) = &self.data {
                return data.clone();
            }
        }

        let prev = self.data.take().and_then(Result::ok);
        let value = result.and_then(|data| f(data, prev));
        self.data = Some(value.clone());

        value
    }
}

/// Read a file.
fn read(path: &Path) -> FileResult<Vec<u8>> {
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

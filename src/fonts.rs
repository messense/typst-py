use std::cell::OnceCell;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use memmap2::Mmap;
use typst::font::{Font, FontBook, FontInfo};
use walkdir::WalkDir;

/// Searches for fonts.
pub struct FontSearcher {
    // Metadata about all discovered fonts.
    pub book: FontBook,
    /// Slots that the fonts are loaded into.
    pub fonts: Vec<FontSlot>,
}

/// Holds details about the location of a font and lazily the font itself.
pub struct FontSlot {
    /// The path at which the font can be found on the system.
    path: PathBuf,
    /// The index of the font in its collection. Zero if the path does not point
    /// to a collection.
    index: u32,
    /// The lazily loaded font.
    font: OnceCell<Option<Font>>,
}

impl FontSlot {
    /// Get the font for this slot.
    pub fn get(&self) -> Option<Font> {
        self.font
            .get_or_init(|| {
                let data = fs::read(&self.path).ok()?.into();
                Font::new(data, self.index)
            })
            .clone()
    }
}

impl FontSearcher {
    /// Create a new, empty system searcher.
    pub fn new() -> Self {
        Self {
            book: FontBook::new(),
            fonts: vec![],
        }
    }

    /// Search everything that is available.
    pub fn search(&mut self, font_paths: &[PathBuf]) {
        for path in font_paths {
            self.search_dir(path)
        }

        self.search_system();
    }

    /// Search for fonts in the linux system font directories.
    fn search_system(&mut self) {
        if cfg!(target_os = "macos") {
            self.search_dir("/Library/Fonts");
            self.search_dir("/Network/Library/Fonts");
            self.search_dir("/System/Library/Fonts");

            // Downloadable fonts, location varies on major macOS releases
            if let Ok(dir) = fs::read_dir("/System/Library/AssetsV2") {
                for entry in dir {
                    let Ok(entry) = entry else { continue };
                    if entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("com_apple_MobileAsset_Font")
                    {
                        self.search_dir(entry.path());
                    }
                }
            }
        } else if cfg!(unix) {
            self.search_dir("/usr/share/fonts");
            self.search_dir("/usr/local/share/fonts");
        } else if cfg!(windows) {
            self.search_dir(
                env::var_os("WINDIR")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| "C:\\Windows".into())
                    .join("Fonts"),
            );

            if let Some(roaming) = dirs::config_dir() {
                self.search_dir(roaming.join("Microsoft\\Windows\\Fonts"));
            }

            if let Some(local) = dirs::cache_dir() {
                self.search_dir(local.join("Microsoft\\Windows\\Fonts"));
            }
        }

        if let Some(dir) = dirs::font_dir() {
            self.search_dir(dir);
        }
    }

    /// Search for all fonts in a directory recursively.
    pub fn search_dir(&mut self, path: impl AsRef<Path>) {
        for entry in WalkDir::new(path)
            .follow_links(true)
            .sort_by(|a, b| a.file_name().cmp(b.file_name()))
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("ttf" | "otf" | "TTF" | "OTF" | "ttc" | "otc" | "TTC" | "OTC"),
            ) {
                self.search_file(path);
            }
        }
    }

    /// Index the fonts in the file at the given path.
    pub fn search_file(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        if let Ok(file) = File::open(path) {
            if let Ok(mmap) = unsafe { Mmap::map(&file) } {
                for (i, info) in FontInfo::iter(&mmap).enumerate() {
                    self.book.push(info);
                    self.fonts.push(FontSlot {
                        path: path.into(),
                        index: i as u32,
                        font: OnceCell::new(),
                    });
                }
            }
        }
    }
}

use honk_engine::tiny_skia::Pixmap;
use std::collections::VecDeque;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const BUILTIN_NOTES: &[&str] = &[
    include_str!("../Assets/Text/NotepadMessages/originals/am goose.txt"),
    include_str!("../Assets/Text/NotepadMessages/originals/good work.txt"),
    include_str!("../Assets/Text/NotepadMessages/originals/gooseASCII1.txt"),
    include_str!("../Assets/Text/NotepadMessages/originals/hard to type.txt"),
    include_str!("../Assets/Text/NotepadMessages/originals/i cause problems.txt"),
    include_str!("../Assets/Text/NotepadMessages/originals/peace was never.txt"),
    include_str!("../Assets/Text/NotepadMessages/custom/custom-am-goose.txt"),
    include_str!("../Assets/Text/NotepadMessages/custom/custom-good-work.txt"),
    include_str!("../Assets/Text/NotepadMessages/custom/custom-gooseASCII1.txt"),
    include_str!("../Assets/Text/NotepadMessages/custom/custom-hard-to-type.txt"),
    include_str!("../Assets/Text/NotepadMessages/custom/custom-i-cause-problems.txt"),
    include_str!("../Assets/Text/NotepadMessages/custom/custom-peace-was-never.txt"),
];

const BUILTIN_MEMES: &[(&str, &[u8])] = &[
    (
        "Meme1",
        include_bytes!("../Assets/Images/Memes/originals/Meme1.png") as &[u8],
    ),
    (
        "Meme3",
        include_bytes!("../Assets/Images/Memes/originals/Meme3.png") as &[u8],
    ),
    (
        "Meme4",
        include_bytes!("../Assets/Images/Memes/originals/Meme4.png") as &[u8],
    ),
    (
        "Meme5",
        include_bytes!("../Assets/Images/Memes/originals/Meme5.png") as &[u8],
    ),
    (
        "Meme6",
        include_bytes!("../Assets/Images/Memes/originals/Meme6.png") as &[u8],
    ),
    (
        "Meme7",
        include_bytes!("../Assets/Images/Memes/originals/Meme7.png") as &[u8],
    ),
    (
        "Meme8",
        include_bytes!("../Assets/Images/Memes/user/Meme8.png") as &[u8],
    ),
    (
        "CustomGooseDance",
        include_bytes!("../Assets/Images/Memes/custom/CustomGooseDance.png") as &[u8],
    ),
    (
        "CustomMeme1",
        include_bytes!("../Assets/Images/Memes/custom/CustomMeme1.png") as &[u8],
    ),
    (
        "CustomMeme2",
        include_bytes!("../Assets/Images/Memes/custom/CustomMeme2.png") as &[u8],
    ),
    (
        "CustomMeme3",
        include_bytes!("../Assets/Images/Memes/custom/CustomMeme3.png") as &[u8],
    ),
    (
        "CustomMeme4",
        include_bytes!("../Assets/Images/Memes/custom/CustomMeme4.png") as &[u8],
    ),
    (
        "CustomMeme5",
        include_bytes!("../Assets/Images/Memes/custom/CustomMeme5.png") as &[u8],
    ),
    (
        "CustomMeme6",
        include_bytes!("../Assets/Images/Memes/custom/CustomMeme6.png") as &[u8],
    ),
    (
        "CustomMeme7",
        include_bytes!("../Assets/Images/Memes/custom/CustomMeme7.png") as &[u8],
    ),
];

const MEME_CACHE_CAPACITY: usize = 2;

pub struct NoteAsset {
    pub text: String,
}

pub struct MemeAsset {
    pub title: String,
    pub pixmap: Pixmap,
}

enum MemeSource {
    BuiltIn(&'static [u8]),
    External(PathBuf),
}

struct MemeEntry {
    title: String,
    width: u32,
    height: u32,
    source: MemeSource,
}

#[derive(Default)]
struct MemeCache {
    entries: VecDeque<(usize, Arc<MemeAsset>)>,
}

pub struct AssetCatalog {
    notes: Vec<NoteAsset>,
    memes: Vec<MemeEntry>,
    cache: Mutex<MemeCache>,
}

impl AssetCatalog {
    pub fn load() -> Self {
        let media_root = user_media_root();
        Self::load_from_media_root(media_root.as_deref())
    }

    fn load_from_media_root(media_root: Option<&Path>) -> Self {
        let mut catalog = Self {
            notes: BUILTIN_NOTES
                .iter()
                .map(|text| NoteAsset {
                    text: (*text).to_owned(),
                })
                .collect(),
            memes: Vec::with_capacity(BUILTIN_MEMES.len()),
            cache: Mutex::new(MemeCache::default()),
        };
        for &(title, bytes) in BUILTIN_MEMES {
            if let Some((width, height)) = png_header(bytes) {
                catalog.memes.push(MemeEntry {
                    title: title.to_owned(),
                    width,
                    height,
                    source: MemeSource::BuiltIn(bytes),
                });
            } else {
                eprintln!("honk300: skipped invalid built-in PNG asset {title}");
            }
        }
        if let Some(media_root) = media_root {
            catalog.load_user_notes(&media_root.join("Notes"));
            catalog.load_user_memes(&media_root.join("Memes"));
        }
        catalog
    }

    pub fn note_count(&self) -> u32 {
        self.notes.len().min(u32::MAX as usize) as u32
    }

    pub fn meme_count(&self) -> u32 {
        self.memes.len().min(u32::MAX as usize) as u32
    }

    pub fn note_text(&self, index: u32) -> Option<&str> {
        self.notes
            .get(index as usize)
            .map(|note| note.text.as_str())
    }

    pub fn meme(&self, index: u32) -> Option<Arc<MemeAsset>> {
        let index = index as usize;
        if index >= self.memes.len() {
            return None;
        }
        {
            let mut cache = self.cache.lock().unwrap_or_else(|err| err.into_inner());
            if let Some(position) = cache
                .entries
                .iter()
                .position(|(cached_index, _)| *cached_index == index)
            {
                let (_, asset) = cache.entries.remove(position).expect("known cache entry");
                cache.entries.push_back((index, Arc::clone(&asset)));
                return Some(asset);
            }
        }

        let entry = &self.memes[index];
        let pixmap = match &entry.source {
            MemeSource::BuiltIn(bytes) => Pixmap::decode_png(bytes).ok(),
            MemeSource::External(path) => fs::read(path)
                .ok()
                .and_then(|bytes| Pixmap::decode_png(&bytes).ok()),
        }?;
        if pixmap.width() != entry.width || pixmap.height() != entry.height {
            eprintln!("honk300: PNG dimensions changed for {}", entry.title);
            return None;
        }
        let asset = Arc::new(MemeAsset {
            title: entry.title.clone(),
            pixmap,
        });
        let mut cache = self.cache.lock().unwrap_or_else(|err| err.into_inner());
        if cache.entries.len() == MEME_CACHE_CAPACITY {
            cache.entries.pop_front();
        }
        cache.entries.push_back((index, Arc::clone(&asset)));
        Some(asset)
    }

    pub fn summary(&self) -> String {
        format!(
            "{} note assets, {} PNG meme assets",
            self.notes.len(),
            self.memes.len()
        )
    }

    fn load_user_notes(&mut self, root: &Path) {
        for path in sorted_files(root) {
            if !has_extension(&path, "txt") {
                continue;
            }
            match fs::read_to_string(&path) {
                Ok(text) if !text.trim().is_empty() => self.notes.push(NoteAsset { text }),
                Ok(_) => {}
                Err(err) => eprintln!("honk300: skipped note asset {} ({err})", path.display()),
            }
        }
    }

    fn load_user_memes(&mut self, root: &Path) {
        for path in sorted_files(root) {
            if !has_extension(&path, "png") {
                continue;
            }
            match read_png_header(&path) {
                Ok((width, height)) => self.memes.push(MemeEntry {
                    title: file_stem(&path),
                    width,
                    height,
                    source: MemeSource::External(path),
                }),
                Err(err) => eprintln!("honk300: skipped meme asset {} ({err})", path.display()),
            }
        }
    }

    #[cfg(test)]
    fn cached_meme_indices(&self) -> Vec<usize> {
        self.cache
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .entries
            .iter()
            .map(|(index, _)| *index)
            .collect()
    }
}

fn has_extension(path: &Path, wanted: &str) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case(wanted))
}

fn read_png_header(path: &Path) -> std::io::Result<(u32, u32)> {
    let mut file = fs::File::open(path)?;
    let mut header = [0u8; 24];
    file.read_exact(&mut header)?;
    png_header(&header)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid PNG header"))
}

fn png_header(bytes: &[u8]) -> Option<(u32, u32)> {
    const SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24
        || &bytes[..8] != SIGNATURE
        || &bytes[12..16] != b"IHDR"
        || u32::from_be_bytes(bytes[8..12].try_into().ok()?) != 13
    {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    (width > 0 && height > 0).then_some((width, height))
}

fn user_media_root() -> Option<PathBuf> {
    if let Some(override_root) = std::env::var_os("HONK300_MEDIA_ROOT") {
        return Some(PathBuf::from(override_root));
    }

    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
        windows_media_root(local_app_data.as_deref())
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME").map(PathBuf::from);
        macos_media_root(home.as_deref())
    }
    #[cfg(target_os = "linux")]
    {
        let xdg_data = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute());
        let home = std::env::var_os("HOME").map(PathBuf::from);
        linux_media_root(xdg_data.as_deref(), home.as_deref())
    }
}

#[cfg(any(test, target_os = "windows"))]
fn windows_media_root(local_app_data: Option<&Path>) -> Option<PathBuf> {
    Some(local_app_data?.join("honk300").join("media"))
}

#[cfg(any(test, target_os = "macos"))]
fn macos_media_root(home: Option<&Path>) -> Option<PathBuf> {
    Some(
        home?
            .join("Library")
            .join("Application Support")
            .join("honk300")
            .join("media"),
    )
}

#[cfg(any(test, target_os = "linux"))]
fn linux_media_root(xdg_data: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    let data_root = xdg_data
        .map(Path::to_path_buf)
        .or_else(|| Some(home?.join(".local").join("share")))?;
    Some(data_root.join("honk300").join("media"))
}

fn sorted_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        files.extend(entries.filter_map(|entry| entry.ok().map(|entry| entry.path())));
    }
    files.sort();
    files
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("untitled")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_do_not_depend_on_an_adjacent_assets_directory() {
        let dir = test_dir("embedded-builtins");
        fs::create_dir_all(&dir).unwrap();

        let catalog = AssetCatalog::load_from_media_root(Some(&dir));

        assert_eq!(catalog.note_count(), 12);
        assert_eq!(catalog.meme_count(), 15);
        assert!(catalog.note_text(0).is_some());
        assert!(catalog.meme(0).is_some());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn approved_meme8_is_an_embedded_builtin() {
        let dir = test_dir("approved-meme8");
        fs::create_dir_all(&dir).unwrap();
        let catalog = AssetCatalog::load_from_media_root(Some(&dir));

        assert!((0..catalog.meme_count())
            .filter_map(|index| catalog.meme(index))
            .any(|meme| meme.title == "Meme8"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn external_user_notes_and_pngs_merge_but_gifs_do_not() {
        let dir = test_dir("user-media");
        let notes = dir.join("Notes");
        let memes = dir.join("Memes");
        fs::create_dir_all(&notes).unwrap();
        fs::create_dir_all(&memes).unwrap();
        fs::write(notes.join("hello.txt"), "hello from user media").unwrap();
        let png = include_bytes!("../Assets/Images/Memes/custom/CustomMeme1.png");
        fs::write(memes.join("my-meme.png"), png).unwrap();
        fs::write(memes.join("animated.gif"), png).unwrap();

        let catalog = AssetCatalog::load_from_media_root(Some(&dir));

        assert_eq!(catalog.note_count(), 13);
        assert_eq!(catalog.meme_count(), 16);
        assert!((0..catalog.note_count())
            .filter_map(|index| catalog.note_text(index))
            .any(|text| text == "hello from user media"));
        assert!((0..catalog.meme_count())
            .filter_map(|index| catalog.meme(index))
            .any(|meme| meme.title == "my-meme"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn decoded_meme_cache_is_lazy_and_bounded_to_two_entries() {
        let dir = test_dir("lazy-cache");
        fs::create_dir_all(&dir).unwrap();
        let catalog = AssetCatalog::load_from_media_root(Some(&dir));
        assert!(catalog.cached_meme_indices().is_empty());

        assert!(catalog.meme(0).is_some());
        assert!(catalog.meme(1).is_some());
        assert_eq!(catalog.cached_meme_indices(), vec![0, 1]);
        assert!(catalog.meme(2).is_some());

        assert_eq!(catalog.cached_meme_indices(), vec![1, 2]);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn platform_media_roots_live_in_user_data_locations() {
        let windows_base = Path::new(r"C:\Users\goose\AppData\Local");
        assert_eq!(
            windows_media_root(Some(windows_base)),
            Some(windows_base.join("honk300").join("media"))
        );
        let macos_home = Path::new("/Users/goose");
        assert_eq!(
            macos_media_root(Some(macos_home)),
            Some(
                macos_home
                    .join("Library")
                    .join("Application Support")
                    .join("honk300")
                    .join("media")
            )
        );
        let xdg_data = Path::new("/data");
        let linux_home = Path::new("/home/goose");
        assert_eq!(
            linux_media_root(Some(xdg_data), Some(linux_home)),
            Some(xdg_data.join("honk300").join("media"))
        );
        assert_eq!(
            linux_media_root(None, Some(linux_home)),
            Some(
                linux_home
                    .join(".local")
                    .join("share")
                    .join("honk300")
                    .join("media")
            )
        );
        assert_eq!(windows_media_root(None), None);
        assert_eq!(macos_media_root(None), None);
        assert_eq!(linux_media_root(None, None), None);
    }

    fn test_dir(name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("honk300-{name}-{}-{nonce}", std::process::id()))
    }
}

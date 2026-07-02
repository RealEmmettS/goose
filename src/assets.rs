use honk_engine::tiny_skia::Pixmap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct NoteAsset {
    pub text: String,
}

pub struct MemeAsset {
    pub title: String,
    pub pixmap: Pixmap,
}

#[derive(Default)]
pub struct AssetCatalog {
    notes: Vec<NoteAsset>,
    memes: Vec<MemeAsset>,
}

impl AssetCatalog {
    pub fn load() -> Self {
        let assets_root = assets_root();
        let mut catalog = Self::default();
        catalog.load_notes(&assets_root.join("Text").join("NotepadMessages"));
        catalog.load_memes(&assets_root.join("Images").join("Memes"));
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

    pub fn meme(&self, index: u32) -> Option<&MemeAsset> {
        self.memes.get(index as usize)
    }

    pub fn summary(&self) -> String {
        format!(
            "{} note assets, {} PNG meme assets",
            self.notes.len(),
            self.memes.len()
        )
    }

    fn load_notes(&mut self, root: &Path) {
        for path in sorted_files(root, &["originals", "custom"]) {
            if path.extension().and_then(|e| e.to_str()) != Some("txt") {
                continue;
            }
            match fs::read_to_string(&path) {
                Ok(text) if !text.trim().is_empty() => self.notes.push(NoteAsset { text }),
                Ok(_) => {}
                Err(err) => eprintln!("honk300: skipped note asset {} ({err})", path.display()),
            }
        }
    }

    fn load_memes(&mut self, root: &Path) {
        for path in sorted_files(root, &["originals", "custom", "user"]) {
            if path.extension().and_then(|e| e.to_str()) != Some("png") {
                continue;
            }
            match Pixmap::load_png(&path) {
                Ok(pixmap) => self.memes.push(MemeAsset {
                    title: file_stem(&path),
                    pixmap,
                }),
                Err(err) => eprintln!("honk300: skipped meme asset {} ({err})", path.display()),
            }
        }
    }
}

fn assets_root() -> PathBuf {
    if let Some(bundle_assets) = bundled_assets_root() {
        return bundle_assets;
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cwd_assets = cwd.join("Assets");
    if cwd_assets.exists() {
        return cwd_assets;
    }

    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("Assets")))
        .filter(|path| path.exists())
        .unwrap_or(cwd_assets)
}

fn bundled_assets_root() -> Option<PathBuf> {
    bundled_assets_root_for_exe(&std::env::current_exe().ok()?)
}

fn bundled_assets_root_for_exe(exe: &Path) -> Option<PathBuf> {
    let bundle = exe
        .ancestors()
        .find(|path| path.extension().and_then(|ext| ext.to_str()) == Some("app"))?;
    let assets = bundle.join("Contents").join("Resources").join("Assets");
    assets.exists().then_some(assets)
}

fn sorted_files(root: &Path, subdirs: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for subdir in subdirs {
        let dir = root.join(subdir);
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
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
    fn detects_assets_inside_macos_app_bundle() {
        let dir = test_dir("bundle-assets");
        let app = dir.join("Honk300.app");
        let assets = app.join("Contents").join("Resources").join("Assets");
        fs::create_dir_all(&assets).unwrap();
        let exe = app.join("Contents").join("MacOS").join("honk300");
        assert_eq!(bundled_assets_root_for_exe(&exe), Some(assets));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn ignores_bare_binary_without_app_bundle() {
        let dir = test_dir("bare-assets");
        fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("honk300");
        assert_eq!(bundled_assets_root_for_exe(&exe), None);
        let _ = fs::remove_dir_all(dir);
    }

    fn test_dir(name: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("honk300-{name}-{}-{nonce}", std::process::id()))
    }
}

//! Versioned sound list and crash-safe persistence for local user data.
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sound {
    pub id: String,
    pub title: String,
    pub path: String,
    pub category: String,
    pub color: String,
    pub favorite: bool,
    pub volume: f32,
    pub destination: Destination,
    #[serde(default)]
    pub duration_seconds: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Destination {
    Monitor,
    Virtual,
    #[default]
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub version: u32,
    pub categories: Vec<String>,
    pub sounds: Vec<Sound>,
}

#[derive(Debug, Clone, Default)]
pub struct LoadResult {
    pub library: Library,
    pub warning: Option<String>,
}

impl Default for Library {
    fn default() -> Self {
        Self {
            version: 1,
            categories: vec![
                "General".into(),
                "Memes".into(),
                "Effects".into(),
                "Voices".into(),
            ],
            sounds: Vec::new(),
        }
    }
}

impl Library {
    /// Starts with an empty library rather than preventing app startup on corrupt data.
    pub fn load_recovering(path: &Path) -> io::Result<LoadResult> {
        if !path.exists() {
            return Ok(LoadResult {
                library: Self::default(),
                warning: None,
            });
        }
        match read_library(path) {
            Ok(library) => Ok(LoadResult {
                library,
                warning: None,
            }),
            Err(error) => {
                let backup = backup_path(path);
                if let Ok(library) = read_library(&backup) {
                    return Ok(LoadResult {
                        library,
                        warning: Some("The sound library was recovered from its backup.".into()),
                    });
                }
                let corrupt = corrupt_path(path);
                fs::rename(path, &corrupt)?;
                Ok(LoadResult {
                    library: Self::default(),
                    warning: Some(format!("The sound library was invalid ({error}) and was saved as {}. A new empty library was created.", corrupt.display())),
                })
            }
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        save_json_atomic(path, self)
    }

    pub fn add(&mut self, sound: Sound) {
        if !self.categories.contains(&sound.category) {
            self.categories.push(sound.category.clone());
        }
        self.sounds.push(sound);
    }
}

pub fn save_json_atomic<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("json.tmp");
    let json = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    let mut file = File::create(&temporary)?;
    file.write_all(&json)?;
    file.sync_all()?;
    drop(file);
    if path.exists() {
        fs::copy(path, backup_path(path))?;
    }
    fs::rename(temporary, path)
}

fn read_library(path: &Path) -> io::Result<Library> {
    let library: Library = serde_json::from_slice(&fs::read(path)?).map_err(io::Error::other)?;
    if library.version != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unsupported library version",
        ));
    }
    Ok(library)
}

fn backup_path(path: &Path) -> PathBuf {
    path.with_extension("json.bak")
}

fn corrupt_path(path: &Path) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    path.with_file_name(format!("library.corrupt-{stamp}.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("anycue-{name}-{stamp}.json"))
    }

    #[test]
    fn initial_library_has_stable_version() {
        assert_eq!(Library::default().version, 1);
    }
    #[test]
    fn destinations_are_versioned() {
        assert!(matches!(
            serde_json::from_str::<Destination>("\"virtual\"").unwrap(),
            Destination::Virtual
        ));
    }
    #[test]
    fn corrupt_primary_recovers_backup() {
        let primary = path("backup");
        fs::write(&primary, b"{").unwrap();
        fs::write(
            backup_path(&primary),
            serde_json::to_vec(&Library::default()).unwrap(),
        )
        .unwrap();
        let result = Library::load_recovering(&primary).unwrap();
        assert_eq!(result.library.version, 1);
        assert!(result.warning.is_some());
        let _ = fs::remove_file(primary);
    }
    #[test]
    fn invalid_and_unknown_versions_start_empty() {
        for content in ["{", "{\"version\":99,\"categories\":[],\"sounds\":[]}"] {
            let primary = path("invalid");
            fs::write(&primary, content).unwrap();
            let result = Library::load_recovering(&primary).unwrap();
            assert!(result.library.sounds.is_empty());
            assert!(result.warning.is_some());
        }
    }
}

//! Versioned sound list. Data stays in a user-owned local directory.
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Sound {
    pub id: String,
    pub title: String,
    pub path: String,
    pub category: String,
    pub color: String,
    pub favorite: bool,
    pub volume: f32,
    pub destination: Destination,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all="lowercase")]
pub enum Destination { Monitor, Virtual, #[default] Both }
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Library {
    pub version: u32,
    pub categories: Vec<String>,
    pub sounds: Vec<Sound>,
}
impl Default for Library {
    fn default() -> Self { Self { version: 1, categories: vec!["Geral".into(),"Memes".into(),"Efeitos".into(),"Vozes".into()],sounds:Vec::new() } }
}
impl Library {
    pub fn load(path: &Path) -> io::Result<Self> {
        if !path.exists() {return Ok(Self::default());}
        let bytes = fs::read(path)?;
        let lib: Self = serde_json::from_slice(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,e))?;
        if lib.version != 1 {return Err(io::Error::new(io::ErrorKind::InvalidData,"Unsupported library version"));}
        Ok(lib)
    }
    pub fn save(&self,path: &Path) -> io::Result<()> {
        if let Some(parent)=path.parent(){fs::create_dir_all(parent)?;}
        let json = serde_json::to_vec_pretty(self)?;
        // Write a new file first. A failed write must never truncate the old library.
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp,json)?;
        if path.exists() { let backup=path.with_extension("json.bak");fs::copy(path,backup)?; fs::remove_file(path)?; }
        fs::rename(tmp,path)
    }
    pub fn add(&mut self,sound: Sound) {
        if !self.categories.contains(&sound.category) {self.categories.push(sound.category.clone());}
        self.sounds.push(sound);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn initial_library_has_stable_version() {assert_eq!(Library::default().version,1);}
    #[test] fn destinations_are_versioned() {let r=serde_json::from_str::<Destination>("\"virtual\"").unwrap();assert!(matches!(r,Destination::Virtual));}
    #[test] fn library_roundtrips() {let original=Library::default();let txt=serde_json::to_string(&original).unwrap();let copy:Library=serde_json::from_str(&txt).unwrap();assert_eq!(copy.categories.len(),4);}
}

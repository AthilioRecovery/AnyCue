//! All CPAL streams live on one audio worker thread: CPAL streams are !Send.
//! Tauri state contains only Send+Sync handles, never an audio stream.
use anycue_audio_io::{AudioEngine, DeviceInfo, Route};
use anycue_library::{Library, Sound};
use std::{
    path::{Path, PathBuf},
    sync::{mpsc::{channel, Receiver, Sender}, Mutex},
    thread,
};
use tauri::{Manager, State};
use uuid::Uuid;

enum AudioRequest {
    Configure { microphone: Option<String>, monitor: Option<String>, virtual_output: Option<String>, reply: Sender<Result<(), String>> },
    Play { path: PathBuf, route: Route, volume: f32, reply: Sender<Result<f64, String>> },
    Volume { value: f32, reply: Sender<Result<(), String>> },
    Stop { reply: Sender<Result<(), String>> },
    Peak { reply: Sender<Result<f32, String>> },
}

struct AppState {
    audio: Sender<AudioRequest>,
    library: Mutex<Library>,
    library_path: PathBuf,
}

fn worker(receiver: Receiver<AudioRequest>) {
    let mut engine: Option<AudioEngine> = None;
    while let Ok(request) = receiver.recv() {
        match request {
            AudioRequest::Configure { microphone, monitor, virtual_output, reply } => {
                let result = AudioEngine::new(microphone.as_deref(), monitor.as_deref(), virtual_output.as_deref());
                let _ = reply.send(match result {
                    Ok(new_engine) => { engine = Some(new_engine); Ok(()) }
                    Err(error) => Err(error),
                });
            }
            AudioRequest::Play { path, route, volume, reply } => {
                let result = engine.as_ref().ok_or_else(|| "Configure audio devices first".to_string())
                    .and_then(|audio| audio.play_file(&path, route, volume));
                let _ = reply.send(result);
            }
            AudioRequest::Volume { value, reply } => {
                if let Some(audio) = engine.as_ref() { audio.set_gain(value); }
                let _ = reply.send(Ok(()));
            }
            AudioRequest::Stop { reply } => {
                if let Some(audio) = engine.as_ref() { audio.stop(); }
                let _ = reply.send(Ok(()));
            }
            AudioRequest::Peak { reply } => {
                let _ = reply.send(Ok(engine.as_ref().map(|audio| audio.peak()).unwrap_or(0.0)));
            }
        }
    }
}
fn ask<T>(state: &AppState, request: impl FnOnce(Sender<Result<T, String>>) -> AudioRequest) -> Result<T, String> {
    let (tx, rx) = channel();
    state.audio.send(request(tx)).map_err(|e| e.to_string())?;
    rx.recv().map_err(|e| e.to_string())?
}
fn lock<'a, T>(mutex: &'a Mutex<T>) -> Result<std::sync::MutexGuard<'a, T>, String> {
    mutex.lock().map_err(|_| "Internal state poisoned".to_string())
}
fn store(state: &AppState, library: &Library) -> Result<(), String> {
    library.save(&state.library_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_devices() -> Result<Vec<DeviceInfo>, String> { anycue_audio_io::devices() }
#[tauri::command]
fn get_library(state: State<'_, AppState>) -> Result<Library, String> { Ok(lock(&state.library)?.clone()) }
#[tauri::command]
fn configure_audio(state: State<'_, AppState>, microphone: Option<String>, monitor: Option<String>, virtual_output: Option<String>) -> Result<(), String> {
    ask(&state, |reply| AudioRequest::Configure { microphone, monitor, virtual_output, reply })
}
#[tauri::command]
fn set_master_volume(state: State<'_, AppState>, volume: f32) -> Result<(), String> {
    ask(&state, |reply| AudioRequest::Volume { value: volume, reply })
}
#[tauri::command]
fn get_peak(state: State<'_, AppState>) -> Result<f32, String> {
    ask(&state, |reply| AudioRequest::Peak { reply })
}
#[tauri::command]
fn stop_all(state: State<'_, AppState>) -> Result<(), String> {
    ask(&state, |reply| AudioRequest::Stop { reply })
}
#[tauri::command]
fn play_sound(state: State<'_, AppState>, id: String) -> Result<f64, String> {
    let sound = lock(&state.library)?.sounds.iter().find(|sound| sound.id == id).cloned().ok_or("Sound not found")?;
    let route = match sound.destination {
        anycue_library::Destination::Monitor => Route::Monitor,
        anycue_library::Destination::Virtual => Route::Virtual,
        anycue_library::Destination::Both => Route::Both,
    };
    ask(&state, |reply| AudioRequest::Play { path: PathBuf::from(sound.path), route, volume: sound.volume, reply })
}
#[tauri::command]
fn add_files(state: State<'_, AppState>, paths: Vec<String>, category: String) -> Result<Library, String> {
    let mut library = lock(&state.library)?;
    for raw in paths.iter().take(200) {
        let path = Path::new(raw);
        if !path.is_file() { continue; }
        let extension = path.extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
        if !["wav", "mp3", "ogg", "flac", "opus", "aac", "m4a"].contains(&extension.as_str()) { continue; }
        let title = path.file_stem().and_then(|value| value.to_str()).unwrap_or("Áudio").to_owned();
        library.add(Sound {
            id: Uuid::new_v4().to_string(), title, path: raw.to_string(), category: category.clone(),
            color: "#b8b8b8".into(), favorite: false, volume: 100.0, destination: Default::default(),
        });
    }
    store(&state, &library)?;
    Ok(library.clone())
}
#[tauri::command]
fn update_sound(state: State<'_, AppState>, sound: Sound) -> Result<(), String> {
    let mut library = lock(&state.library)?;
    let existing = library.sounds.iter_mut().find(|entry| entry.id == sound.id).ok_or("Sound not found")?;
    // An editor request must not rewrite the original file path.
    let mut amended = sound;
    amended.path = existing.path.clone();
    amended.volume = amended.volume.clamp(0.0, 200.0);
    amended.title = amended.title.chars().take(80).collect();
    *existing = amended;
    store(&state, &library)
}
#[tauri::command]
fn delete_sound(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut library = lock(&state.library)?;
    library.sounds.retain(|entry| entry.id != id);
    store(&state, &library)
}
#[tauri::command]
fn add_category(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let name: String = name.trim().chars().take(25).collect();
    if name.is_empty() { return Err("Category name is empty".into()); }
    let mut library = lock(&state.library)?;
    if !library.categories.contains(&name) { library.categories.push(name); store(&state, &library)?; }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            let library_path = directory.join("library.json");
            let library = Library::load(&library_path)?;
            let (sender, receiver) = channel();
            thread::Builder::new().name("anycue-audio".into()).spawn(move || worker(receiver))?;
            app.manage(AppState { audio: sender, library: Mutex::new(library), library_path });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_devices, get_library, configure_audio, set_master_volume, get_peak,
            stop_all, play_sound, add_files, update_sound, delete_sound, add_category
        ])
        .run(tauri::generate_context!())
        .expect("Failed to start AnyCue");
}
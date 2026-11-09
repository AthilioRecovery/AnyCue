use anycue_audio_io::{AudioEngine,DeviceInfo,Route};
use anycue_library::{Library,Sound};
use std::{path::{Path,PathBuf},sync::Mutex};
use tauri::{Manager,State};
use uuid::Uuid;

struct AppState{engine:Mutex<Option<AudioEngine>>,library:Mutex<Library>,library_path:PathBuf}
fn lock<'a,T>(mutex:&'a Mutex<T>)->Result<std::sync::MutexGuard<'a,T>,String>{mutex.lock().map_err(|_|"Internal state poisoned".to_string())}
fn store(state:&AppState,lib:&Library)->Result<(),String>{lib.save(&state.library_path).map_err(|e|e.to_string())}

#[tauri::command]
fn list_devices()->Result<Vec<DeviceInfo>,String>{anycue_audio_io::devices()}
#[tauri::command]
fn get_library(state:State<'_,AppState>)->Result<Library,String>{Ok(lock(&state.library)?.clone())}
#[tauri::command]
fn configure_audio(state:State<'_,AppState>,microphone:Option<String>,monitor:Option<String>,virtual_output:Option<String>)->Result<(),String>{
    let audio=AudioEngine::new(microphone.as_deref(),monitor.as_deref(),virtual_output.as_deref())?;
    *lock(&state.engine)?=Some(audio);
    Ok(())
}
#[tauri::command]
fn set_master_volume(state:State<'_,AppState>,volume:f32)->Result<(),String>{if let Some(engine)=lock(&state.engine)?.as_ref(){engine.set_gain(volume);}Ok(())}
#[tauri::command]
fn get_peak(state:State<'_,AppState>)->Result<f32,String>{Ok(lock(&state.engine)?.as_ref().map(|e|e.peak()).unwrap_or(0.0))}
#[tauri::command]
fn stop_all(state:State<'_,AppState>)->Result<(),String>{if let Some(engine)=lock(&state.engine)?.as_ref(){engine.stop();}Ok(())}
#[tauri::command]
fn play_sound(state:State<'_,AppState>,id:String)->Result<f64,String>{
    let sound=lock(&state.library)?.sounds.iter().find(|s|s.id==id).cloned().ok_or("Sound not found")?;
    let route=match sound.destination {anycue_library::Destination::Monitor=>Route::Monitor,anycue_library::Destination::Virtual=>Route::Virtual,anycue_library::Destination::Both=>Route::Both};
    let guard=lock(&state.engine)?;
    let engine=guard.as_ref().ok_or("Audio device not configured")?;
    engine.play_file(Path::new(&sound.path),route,sound.volume)
}
#[tauri::command]
fn add_files(state:State<'_,AppState>,paths:Vec<String>,category:String)->Result<Library,String>{
    let mut library=lock(&state.library)?;
    for raw in paths.iter().take(200){
        let path=Path::new(raw);
        if !path.is_file(){continue;}
        let ext=path.extension().and_then(|x|x.to_str()).unwrap_or("").to_ascii_lowercase();
        if !["wav","mp3","ogg","flac","opus","aac","m4a"].contains(&ext.as_str()){continue;}
        let title=path.file_stem().and_then(|x|x.to_str()).unwrap_or("Áudio").to_owned();
        library.add(Sound{id:Uuid::new_v4().to_string(),title,path:raw.to_string(),category:category.clone(),color:"#8ea8ff".into(),favorite:false,volume:100.0,destination:Default::default()});
    }
    store(&state,&library)?;
    Ok(library.clone())
}
#[tauri::command]
fn update_sound(state:State<'_,AppState>,sound:Sound)->Result<(),String>{
    let mut lib=lock(&state.library)?;
    let existing=lib.sounds.iter_mut().find(|s|s.id==sound.id).ok_or("Sound not found")?;
    // A UI edit cannot replace the path to an arbitrary file.
    let mut amended=sound;amended.path=existing.path.clone();
    amended.volume=amended.volume.clamp(0.0,200.0);
    amended.title=amended.title.chars().take(80).collect();
    *existing=amended;
    store(&state,&lib)
}
#[tauri::command]
fn delete_sound(state:State<'_,AppState>,id:String)->Result<(),String>{
    let mut lib=lock(&state.library)?;
    lib.sounds.retain(|s|s.id!=id);store(&state,&lib)
}
#[tauri::command]
fn add_category(state:State<'_,AppState>,name:String)->Result<(),String>{
    let name:String=name.trim().chars().take(25).collect();
    if name.is_empty(){return Err("Category name is empty".into());}
    let mut lib=lock(&state.library)?;
    if !lib.categories.contains(&name){lib.categories.push(name);store(&state,&lib)?;}
    Ok(())
}

#[cfg_attr(mobile,tauri::mobile_entry_point)]
pub fn run(){
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app|{
            let dir=app.path().app_data_dir()?;
            let file=dir.join("library.json");
            let library=Library::load(&file)?;
            app.manage(AppState{engine:Mutex::new(None),library:Mutex::new(library),library_path:file});
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_devices,get_library,configure_audio,set_master_volume,get_peak,stop_all,play_sound,add_files,update_sound,delete_sound,add_category])
        .run(tauri::generate_context!())
        .expect("Failed to start AnyCue");
}

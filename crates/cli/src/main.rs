//! Minimal no-UI desktop audio commands. Windows + WASAPI through CPAL.
use anycue_audio_io::{devices, AudioEngine, Route};
use std::{env, path::Path, process, thread, time::Duration};
fn main() {
    if let Err(error) = run() {
        eprintln!("AnyCue: {error}");
        process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("devices") => {
            for d in devices()? {
                println!(
                    "{}\t{}{}",
                    d.direction,
                    d.name,
                    if d.is_default { " [default]" } else { "" }
                );
            }
            Ok(())
        }
        Some("doctor") => {
            let list = devices()?;
            println!("AnyCue CLI 0.1-alpha");
            println!(
                "Input devices: {}",
                list.iter().filter(|d| d.direction == "input").count()
            );
            println!(
                "Output devices: {}",
                list.iter().filter(|d| d.direction == "output").count()
            );
            let has_cable = list.iter().any(|d| {
                d.name.to_lowercase().contains("cable")
                    || d.name.to_lowercase().contains("voicemeeter")
            });
            println!("Known virtual cable name found: {}", has_cable);
            println!("This check does not prove a virtual cable is functioning.");
            Ok(())
        }
        Some("play") => {
            let file = args.next().ok_or(
                "Usage: anycue play sound.mp3 [--monitor NAME] [--virtual NAME] [--mic NAME]",
            )?;
            let mut monitor = None;
            let mut virtual_name = None;
            let mut mic = None;
            while let Some(flag) = args.next() {
                let value = args.next().ok_or(format!("Missing value for {flag}"))?;
                match flag.as_str() {
                    "--monitor" => monitor = Some(value),
                    "--virtual" => virtual_name = Some(value),
                    "--mic" => mic = Some(value),
                    _ => return Err(format!("Unknown flag: {flag}")),
                }
            }
            if monitor.is_none() && virtual_name.is_none() {
                monitor = devices()?
                    .into_iter()
                    .find(|d| d.direction == "output" && d.is_default)
                    .map(|d| d.name);
            }
            let engine =
                AudioEngine::new(mic.as_deref(), monitor.as_deref(), virtual_name.as_deref())?;
            let route = match (monitor.is_some(), virtual_name.is_some()) {
                (true, true) => Route::Both,
                (true, false) => Route::Monitor,
                (false, true) => Route::Virtual,
                _ => return Err("No playback device".into()),
            };
            let duration = engine.play_file(Path::new(&file), route, 100.0)?;
            println!("Playing {file} ({duration:.1}s). Ctrl+C to stop.");
            thread::sleep(Duration::from_secs_f64(duration + 0.5));
            Ok(())
        }
        _ => {
            println!("AnyCue open-source soundboard CLI\n\nUsage:\n  anycue devices\n  anycue doctor\n  anycue play <file> [--monitor NAME] [--virtual NAME] [--mic NAME]\n\nNote: commands for a background daemon are not implemented in the alpha.");
            Ok(())
        }
    }
}

//! Local WASAPI output using CPAL. Works with VB-CABLE or another Windows
//! virtual playback device. No driver is installed by this application.
mod decode;
use anycue_core::{gain_from_percent, mix_stereo};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_queue::ArrayQueue;
pub use decode::{decode_file, Clip};
use std::{
    path::Path,
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
};

pub trait VirtualMicBackend {
    fn backend_description(&self) -> &'static str;
}
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub name: String,
    pub direction: String,
    pub is_default: bool,
}

pub fn devices() -> Result<Vec<DeviceInfo>, String> {
    let host = cpal::default_host();
    let mut out = Vec::new();
    let default_input = host.default_input_device().and_then(|d| d.name().ok());
    let default_output = host.default_output_device().and_then(|d| d.name().ok());
    for device in host.input_devices().map_err(|e| e.to_string())? {
        if let Ok(name) = device.name() {
            out.push(DeviceInfo {
                is_default: Some(&name) == default_input.as_ref(),
                name,
                direction: "input".into(),
            });
        }
    }
    for device in host.output_devices().map_err(|e| e.to_string())? {
        if let Ok(name) = device.name() {
            out.push(DeviceInfo {
                is_default: Some(&name) == default_output.as_ref(),
                name,
                direction: "output".into(),
            });
        }
    }
    Ok(out)
}

#[derive(Clone, Copy, Debug)]
pub enum Route {
    Monitor,
    Virtual,
    Both,
}
#[derive(Clone)]
enum Command {
    Play { clip: Arc<Clip>, volume: f32 },
    Stop,
    Gain(f32),
}
struct Voice {
    clip: Arc<Clip>,
    cursor: f64,
    volume: f32,
}
struct OutputState {
    commands: Receiver<Command>,
    voices: Vec<Voice>,
    rate: u32,
    master: f32,
    microphone: Option<Arc<ArrayQueue<f32>>>,
    peak: Arc<AtomicU32>,
}
impl OutputState {
    fn frame(&mut self) -> [f32; 2] {
        while let Ok(message) = self.commands.try_recv() {
            match message {
                Command::Play { clip, volume } => {
                    if self.voices.len() < 24 {
                        self.voices.push(Voice {
                            clip,
                            cursor: 0.0,
                            volume,
                        });
                    }
                }
                Command::Stop => self.voices.clear(),
                Command::Gain(v) => self.master = gain_from_percent(v),
            }
        }
        let mut audio = [0.0f32; 2];
        self.voices.retain_mut(|voice| {
            let i = voice.cursor as usize;
            if i >= voice.clip.frames.len() {
                return false;
            }
            let next = (i + 1).min(voice.clip.frames.len() - 1);
            let mix = (voice.cursor - i as f64) as f32;
            for (channel, dest) in audio.iter_mut().enumerate() {
                let current = voice.clip.frames[i][channel];
                let following = voice.clip.frames[next][channel];
                *dest += (current + (following - current) * mix) * voice.volume;
            }
            voice.cursor += voice.clip.rate as f64 / self.rate as f64;
            true
        });
        let mic = self
            .microphone
            .as_ref()
            .and_then(|r| r.pop())
            .unwrap_or(0.0);
        let mixed = mix_stereo([mic, mic], audio, 1.0, self.master);
        let peak = mixed[0].abs().max(mixed[1].abs());
        self.peak.store(peak.to_bits(), Ordering::Relaxed);
        mixed
    }
}
struct OutputHandle {
    sender: Sender<Command>,
    _stream: cpal::Stream,
    peak: Arc<AtomicU32>,
}
impl OutputHandle {
    fn play(&self, clip: Arc<Clip>, volume: f32) -> Result<(), String> {
        self.sender
            .send(Command::Play { clip, volume })
            .map_err(|e| e.to_string())
    }
    fn stop(&self) {
        let _ = self.sender.send(Command::Stop);
    }
    fn gain(&self, value: f32) {
        let _ = self.sender.send(Command::Gain(value));
    }
}

pub struct AudioEngine {
    monitor: Option<OutputHandle>,
    virtual_output: Option<OutputHandle>,
    _input: Option<cpal::Stream>,
}
impl VirtualMicBackend for AudioEngine {
    fn backend_description(&self) -> &'static str {
        "WASAPI shared-mode through CPAL; external virtual cable"
    }
}
impl AudioEngine {
    /// Names are the user-selected device labels obtained from `devices`.
    /// None for a route disables it; both none is rejected.
    pub fn new(
        input_name: Option<&str>,
        monitor_name: Option<&str>,
        virtual_name: Option<&str>,
    ) -> Result<Self, String> {
        if monitor_name.is_none() && virtual_name.is_none() {
            return Err("Choose a monitor or virtual output device".into());
        }
        let ring = Arc::new(ArrayQueue::new(48_000));
        let microphone = if virtual_name.is_some() {
            match input_name {
                Some(name) => Some(create_input(name, ring.clone())?),
                None => None,
            }
        } else {
            None
        };
        let monitor = match monitor_name {
            Some(name) => Some(create_output(name, None)?),
            None => None,
        };
        let virtual_output = match virtual_name {
            Some(name) => Some(create_output(
                name,
                if microphone.is_some() {
                    Some(ring)
                } else {
                    None
                },
            )?),
            None => None,
        };
        Ok(Self {
            monitor,
            virtual_output,
            _input: microphone,
        })
    }
    pub fn play_file(&self, path: &Path, route: Route, volume: f32) -> Result<f64, String> {
        let clip = decode_file(path)?;
        let duration = clip.duration();
        let v = gain_from_percent(volume);
        if matches!(route, Route::Both | Route::Monitor) {
            if let Some(output) = &self.monitor {
                output.play(clip.clone(), v)?;
            }
        }
        if matches!(route, Route::Both | Route::Virtual) {
            if let Some(output) = &self.virtual_output {
                output.play(clip, v)?;
            }
        }
        Ok(duration)
    }
    pub fn stop(&self) {
        if let Some(o) = &self.monitor {
            o.stop();
        }
        if let Some(o) = &self.virtual_output {
            o.stop();
        }
    }
    pub fn set_gain(&self, value: f32) {
        if let Some(o) = &self.monitor {
            o.gain(value);
        }
        if let Some(o) = &self.virtual_output {
            o.gain(value);
        }
    }
    pub fn peak(&self) -> f32 {
        let a = self
            .monitor
            .as_ref()
            .map(|o| f32::from_bits(o.peak.load(Ordering::Relaxed)))
            .unwrap_or(0.0);
        let b = self
            .virtual_output
            .as_ref()
            .map(|o| f32::from_bits(o.peak.load(Ordering::Relaxed)))
            .unwrap_or(0.0);
        a.max(b)
    }
}
fn get_device(name: &str, input: bool) -> Result<cpal::Device, String> {
    let host = cpal::default_host();
    let devices: Vec<_> = if input {
        host.input_devices().map_err(|e| e.to_string())?.collect()
    } else {
        host.output_devices().map_err(|e| e.to_string())?.collect()
    };
    devices
        .into_iter()
        .find(|d| d.name().ok().as_deref() == Some(name))
        .ok_or_else(|| format!("Audio device missing or unplugged: {name}"))
}
fn create_output(
    name: &str,
    microphone: Option<Arc<ArrayQueue<f32>>>,
) -> Result<OutputHandle, String> {
    let device = get_device(name, false)?;
    let supported = device.default_output_config().map_err(|e| e.to_string())?;
    let sample_format = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();
    let (sender, receiver) = channel();
    let peak = Arc::new(AtomicU32::new(0));
    let state = OutputState {
        commands: receiver,
        voices: Vec::with_capacity(24),
        rate: config.sample_rate.0,
        master: 1.0,
        microphone,
        peak: peak.clone(),
    };
    let channels = config.channels as usize;
    let err = |e| eprintln!("AnyCue audio output error: {e}");
    let stream = match sample_format {
        cpal::SampleFormat::F32 => {
            let mut s = state;
            device.build_output_stream(
                &config,
                move |data: &mut [f32], _| render(data, channels, &mut s, |v| v),
                err,
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            let mut s = state;
            device.build_output_stream(
                &config,
                move |data: &mut [i16], _| {
                    render(data, channels, &mut s, |v| {
                        (v.clamp(-1.0, 1.0) * 32767.0) as i16
                    })
                },
                err,
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let mut s = state;
            device.build_output_stream(
                &config,
                move |data: &mut [u16], _| {
                    render(data, channels, &mut s, |v| {
                        ((v.clamp(-1.0, 1.0) + 1.0) * 32767.5) as u16
                    })
                },
                err,
                None,
            )
        }
        other => return Err(format!("Unsupported output sample format: {other:?}")),
    }
    .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    Ok(OutputHandle {
        sender,
        _stream: stream,
        peak,
    })
}
fn render<T: Copy>(
    output: &mut [T],
    channels: usize,
    state: &mut OutputState,
    convert: impl Fn(f32) -> T,
) {
    for frame in output.chunks_mut(channels) {
        let mixed = state.frame();
        for (i, sample) in frame.iter_mut().enumerate() {
            *sample = convert(mixed[i.min(1)]);
        }
    }
}
fn create_input(name: &str, ring: Arc<ArrayQueue<f32>>) -> Result<cpal::Stream, String> {
    let device = get_device(name, true)?;
    let supported = device.default_input_config().map_err(|e| e.to_string())?;
    let sample_format = supported.sample_format();
    let config: cpal::StreamConfig = supported.into();
    let channels = config.channels as usize;
    let err = |e| eprintln!("AnyCue input error: {e}");
    let stream = match sample_format {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _| capture(data, channels, &ring, |v| v),
            err,
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _| capture(data, channels, &ring, |v| v as f32 / 32768.0),
            err,
            None,
        ),
        cpal::SampleFormat::U16 => device.build_input_stream(
            &config,
            move |data: &[u16], _| {
                capture(data, channels, &ring, |v| (v as f32 - 32768.0) / 32768.0)
            },
            err,
            None,
        ),
        other => return Err(format!("Unsupported input sample format: {other:?}")),
    }
    .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    Ok(stream)
}
fn capture<T: Copy>(
    input: &[T],
    channels: usize,
    ring: &Arc<ArrayQueue<f32>>,
    convert: impl Fn(T) -> f32,
) {
    for frame in input.chunks_exact(channels) {
        let mut v = 0.0f32;
        for sample in frame {
            v += convert(*sample);
        }
        v /= channels as f32;
        if ring.push(v).is_err() {
            let _ = ring.pop();
            let _ = ring.push(v);
        }
    }
}

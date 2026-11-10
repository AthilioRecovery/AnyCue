//! Blocking decode on the caller thread. NEVER decode inside audio callback.
use std::{
    collections::HashMap,
    fs::{self, File},
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};
use symphonia::{
    core::{
        audio::SampleBuffer, codecs::DecoderOptions, errors::Error as SymphoniaError,
        formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
    },
    default::{get_codecs, get_probe},
};

#[derive(Debug)]
pub struct Clip {
    pub frames: Vec<[f32; 2]>,
    pub rate: u32,
}

pub const DEFAULT_CLIP_CACHE_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileRevision {
    len: u64,
    modified: Option<SystemTime>,
}

impl FileRevision {
    fn read(path: &Path) -> Result<Self, String> {
        let metadata = fs::metadata(path)
            .map_err(|error| format!("Audio file is missing or unreadable: {error}"))?;
        Ok(Self {
            len: metadata.len(),
            modified: metadata.modified().ok(),
        })
    }
}

struct CacheEntry {
    clip: Arc<Clip>,
    revision: FileRevision,
    used_at: u64,
}

/// A worker-thread-only LRU cache. It is deliberately not synchronized because
/// decoding and cache access happen outside both the UI and audio callbacks.
pub struct DecodedClipCache {
    entries: HashMap<PathBuf, CacheEntry>,
    capacity_bytes: usize,
    used_bytes: usize,
    clock: u64,
}

impl DecodedClipCache {
    pub fn new(capacity_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            capacity_bytes,
            used_bytes: 0,
            clock: 0,
        }
    }

    pub fn get_or_decode(&mut self, path: &Path) -> Result<Arc<Clip>, String> {
        let revision = FileRevision::read(path)?;
        self.clock = self.clock.wrapping_add(1);
        if let Some(entry) = self.entries.get_mut(path) {
            if entry.revision == revision {
                entry.used_at = self.clock;
                return Ok(entry.clip.clone());
            }
        }
        self.remove(path);
        let clip = decode_file(path)?;
        let bytes = clip
            .frames
            .len()
            .saturating_mul(std::mem::size_of::<[f32; 2]>());
        if bytes > self.capacity_bytes {
            return Err(format!("Audio needs {bytes} bytes after decoding, exceeding the configured cache limit of {} bytes. Long-file streaming is not implemented yet.", self.capacity_bytes));
        }
        while self.used_bytes.saturating_add(bytes) > self.capacity_bytes {
            let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.used_at)
                .map(|(path, _)| path.clone())
            else {
                break;
            };
            self.remove(&oldest);
        }
        self.used_bytes = self.used_bytes.saturating_add(bytes);
        self.entries.insert(
            path.to_path_buf(),
            CacheEntry {
                clip: clip.clone(),
                revision,
                used_at: self.clock,
            },
        );
        Ok(clip)
    }

    pub fn remove(&mut self, path: &Path) {
        if let Some(entry) = self.entries.remove(path) {
            self.used_bytes = self.used_bytes.saturating_sub(
                entry
                    .clip
                    .frames
                    .len()
                    .saturating_mul(std::mem::size_of::<[f32; 2]>()),
            );
        }
    }
}
impl Clip {
    pub fn duration(&self) -> f64 {
        self.frames.len() as f64 / self.rate as f64
    }
}
pub fn decode_file(path: &Path) -> Result<Arc<Clip>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(ext);
    }
    let probed = get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| e.to_string())?;
    let mut format = probed.format;
    let track = format.default_track().ok_or("No audio track")?;
    let id = track.id;
    let params = track.codec_params.clone();
    let mut decoder = get_codecs()
        .make(&params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;
    let rate = params.sample_rate.ok_or("Unknown sample rate")?;
    const MAX_DECODED_BYTES: usize = 256 * 1024 * 1024;
    const BYTES_PER_STEREO_FRAME: usize = std::mem::size_of::<[f32; 2]>();
    let mut frames = Vec::new();
    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(_)) => break,
            Err(e) => return Err(e.to_string()),
        };
        if packet.track_id() != id {
            continue;
        }
        let buffer = match decoder.decode(&packet) {
            Ok(v) => v,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::IoError(_)) => break,
            Err(e) => return Err(e.to_string()),
        };
        let channels = buffer.spec().channels.count();
        let frame_count = buffer.frames();
        let spec = *buffer.spec();
        let mut samples = SampleBuffer::<f32>::new(frame_count as u64, spec);
        samples.copy_interleaved_ref(buffer);
        for frame in samples.samples().chunks_exact(channels) {
            if frames.len() >= MAX_DECODED_BYTES / BYTES_PER_STEREO_FRAME {
                return Err("Audio is too large to decode in memory (limit: 256 MiB). Long files require streaming support, which is not yet available.".into());
            }
            frames.push([frame[0], if channels > 1 { frame[1] } else { frame[0] }]);
        }
    }
    if frames.is_empty() {
        return Err("Empty or unsupported audio file".into());
    }
    Ok(Arc::new(Clip { frames, rate }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn wav(name: &str, samples: &[i16]) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("anycue-cache-{name}-{stamp}.wav"));
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 48_000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&path, spec).unwrap();
        for sample in samples {
            writer.write_sample(*sample).unwrap();
        }
        writer.finalize().unwrap();
        path
    }

    #[test]
    fn cache_returns_same_clip_for_unchanged_file() {
        let path = wav("hit", &[1, 2, 3, 4]);
        let mut cache = DecodedClipCache::new(1024);
        let first = cache.get_or_decode(&path).unwrap();
        let second = cache.get_or_decode(&path).unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn cache_evicts_least_recent_clip_under_limit() {
        let first_path = wav("first", &[1, 2, 3, 4]);
        let second_path = wav("second", &[5, 6, 7, 8]);
        let mut cache = DecodedClipCache::new(32);
        let first = cache.get_or_decode(&first_path).unwrap();
        let _ = cache.get_or_decode(&second_path).unwrap();
        let reloaded = cache.get_or_decode(&first_path).unwrap();
        assert!(!Arc::ptr_eq(&first, &reloaded));
        let _ = fs::remove_file(first_path);
        let _ = fs::remove_file(second_path);
    }

    #[test]
    fn cache_invalidates_changed_file() {
        let path = wav("changed", &[1, 2, 3, 4]);
        let mut cache = DecodedClipCache::new(1024);
        let first = cache.get_or_decode(&path).unwrap();
        let replacement = wav("replacement", &[1, 2, 3, 4, 5]);
        fs::copy(replacement, &path).unwrap();
        let second = cache.get_or_decode(&path).unwrap();
        assert!(!Arc::ptr_eq(&first, &second));
        let _ = fs::remove_file(path);
    }
}

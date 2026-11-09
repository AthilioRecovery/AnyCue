//! Blocking decode on the caller thread. NEVER decode inside audio callback.
use std::{fs::File, path::Path, sync::Arc};
use symphonia::{core::{audio::SampleBuffer,codecs::DecoderOptions,errors::Error as SymphoniaError,formats::FormatOptions,io::MediaSourceStream,meta::MetadataOptions,probe::Hint},default::{get_codecs,get_probe}};

#[derive(Debug)]
pub struct Clip {pub frames: Vec<[f32;2]>,pub rate:u32}
impl Clip {pub fn duration(&self)->f64{self.frames.len() as f64 / self.rate as f64}}
pub fn decode_file(path:&Path)->Result<Arc<Clip>,String>{
    let file=File::open(path).map_err(|e|e.to_string())?;
    let mss=MediaSourceStream::new(Box::new(file),Default::default());
    let mut hint=Hint::new();if let Some(ext)=path.extension().and_then(|s|s.to_str()){hint.with_extension(ext);}
    let probed=get_probe().format(&hint,mss,&FormatOptions::default(),&MetadataOptions::default()).map_err(|e|e.to_string())?;
    let mut format=probed.format;
    let track=format.default_track().ok_or("No audio track")?;
    let id=track.id;let params=track.codec_params.clone();
    let mut decoder=get_codecs().make(&params,&DecoderOptions::default()).map_err(|e|e.to_string())?;
    let rate=params.sample_rate.ok_or("Unknown sample rate")?;
    let mut frames=Vec::new();
    loop {
        let packet=match format.next_packet(){Ok(p)=>p,Err(SymphoniaError::IoError(_))=>break,Err(e)=>return Err(e.to_string())};
        if packet.track_id()!=id{continue;}
        let buffer=match decoder.decode(&packet){Ok(v)=>v,Err(SymphoniaError::DecodeError(_))=>continue,Err(SymphoniaError::IoError(_))=>break,Err(e)=>return Err(e.to_string())};
        let channels=buffer.spec().channels.count();
        let frame_count=buffer.frames();
        let spec=*buffer.spec();
        let mut samples=SampleBuffer::<f32>::new(frame_count as u64,spec);
        samples.copy_interleaved_ref(buffer);
        for frame in samples.samples().chunks_exact(channels){frames.push([frame[0],if channels>1{frame[1]}else{frame[0]}]);}
        if frames.len()>60*60*192000{return Err("Audio exceeds 60 minutes".into());}
    }
    if frames.is_empty(){return Err("Empty or unsupported audio file".into());}
    Ok(Arc::new(Clip{frames,rate}))
}

// src/audio.rs
use rodio::{OutputStream, OutputStreamHandle, Sink}; // Assure-toi d'avoir ces imports
use std::path::Path;

pub struct AudioEngine {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink,
}

impl AudioEngine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        Ok(Self { _stream: stream, stream_handle, sink })
    }

    pub fn play_file(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.sink.stop(); // Arrête la piste précédente
        let file = std::fs::File::open(path)?;
        let source = rodio::Decoder::new(std::io::BufReader::new(file))?;
        self.sink.append(source);
        Ok(())
    }

    pub fn toggle_pause(&self) {
        if self.sink.is_paused() { self.sink.play(); }
        else { self.sink.pause(); }
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
}
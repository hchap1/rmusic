use std::fs::File;
use std::io::BufReader;
use rodio::Sink;
use rodio::{Decoder, OutputStream, OutputStreamHandle};

use crate::downloader::Song;

pub struct AudioPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        Self {
            _stream: stream,
            stream_handle,
            sink
        }
    }

    pub fn play(&mut self, song: Song) {
        if song.file == None { return; }
        let file = BufReader::new(File::open(song.file.unwrap()).unwrap());
        let source = Decoder::new(file).unwrap();
        self.sink.append(source);
    }

    pub fn pause(&mut self) {
        self.sink.pause();
    }

    pub fn resume(&mut self) {
        self.sink.play();
    }
}

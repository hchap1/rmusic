use std::fs::File;
use std::io::BufReader;
use rodio::Sink;
use rodio::{Decoder, OutputStream, OutputStreamHandle};
use std::sync::{
    Arc,
    Mutex
};
use std::time::Duration;
use std::thread::{
    sleep,
    JoinHandle,
    spawn
};

use crate::downloader::Song;
type AMV<T> = Arc<Mutex<Vec<T>>>;
type AM<T> = Arc<Mutex<T>>;

fn sync<T>(obj: T) -> AM<T> { Arc::new(Mutex::new(obj)) }

pub struct AudioPlayer {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sink: AM<Sink>,
    playlist: AMV<Song>,
    _poll_handle: JoinHandle<()>,
    queue_sink_clear: AM<bool>
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = sync(Sink::try_new(&stream_handle).unwrap());

        let playlist: AMV<Song> = sync(Vec::new());
        let polling_sink = Arc::clone(&sink);
        let polling_playlist = Arc::clone(&playlist);
        let queue_clear = sync(false);
        let polling_clear = Arc::clone(&queue_clear);

        let _poll_handle = spawn(move || {
            manage_queue(polling_playlist, polling_sink, polling_clear);
        });
        
        Self {
            _stream: stream,
            _stream_handle: stream_handle,
            sink,
            playlist,
            _poll_handle,
            queue_sink_clear: queue_clear
        }
    }

    pub fn play(&mut self, song: Song) {
        let mut playlist = self.playlist.lock().unwrap();
        playlist.clear();
        playlist.push(song.clone());
        let mut queue_clear = self.queue_sink_clear.lock().unwrap();
        *queue_clear = true;
    }

    pub fn pause(&mut self) {
        let sink = self.sink.lock().unwrap();
        sink.pause();
    }

    pub fn resume(&mut self) {
        let sink = self.sink.lock().unwrap();
        sink.play();
    }

    pub fn toggle(&mut self) {
        let running = {
            let sink = self.sink.lock().unwrap();
            !sink.is_paused()
        };

        if running { self.pause(); }
        else { self.resume(); }
    }

    pub fn get_queue(&mut self) -> Vec<String> {
        let queue = self.playlist.lock().unwrap();
        queue.iter().map(|song| song.name.clone()).collect::<Vec<String>>()
    }

    pub fn skip(&mut self) {
        let sink = self.sink.lock().unwrap();
        let mut queue = self.playlist.lock().unwrap();

        if queue.len() > 0 {
            queue.remove(0);
        }
        sink.skip_one();
    }

    pub fn append(&mut self, song: Song) {
        let mut queue = self.playlist.lock().unwrap();
        queue.push(song.clone());
    }
}

pub fn manage_queue(queue: AMV<Song>, sink: AM<Sink>, clear: AM<bool>) {
    loop {
        sleep(Duration::from_secs(1));
        {
            let sink = sink.lock().unwrap();
            let mut c = clear.lock().unwrap();

            if *c {
                sink.clear();
                *c = false;
                sink.play();
            }

            if sink.empty() {
                let queue = queue.lock().unwrap();
                if queue.len() < 1 {
                    continue;
                }
                let file = BufReader::new(File::open(queue[0].file.clone().unwrap()).unwrap());
                let source = Decoder::new(file).unwrap();
                sink.append(source);
            }
        }
    }
}

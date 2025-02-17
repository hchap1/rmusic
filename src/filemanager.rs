use std::collections::HashSet;
use std::fs::read_to_string;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::fs::read_dir;
use std::io::Write;

use crate::downloader::Song;

fn find_smallest_unused_id(dir: PathBuf) -> Result<Vec<usize>, ()> {
    let contents = match read_dir(dir) {
        Ok(contents) => contents,
        Err(_) => return Err(())
    };

    for entry in contents {
        let item = match entry {
            Ok(entry) => entry,
            Err(_) => continue
        };

        let filename = item.file_name().to_string_lossy().to_string();
        println!("File: {filename}");
    }

    Ok(Vec::new())
}

pub struct Playlist {
    pub songs: Vec<Song>,
    collected: HashSet<String>,
    file: PathBuf
}

impl Playlist {
    pub fn load_playlist(file: PathBuf) -> Result<Self, ()> {
        // Playlist file structured as song||artist||url||file abs path OR _ if not downloaded
        
        let contents = match read_to_string(&file) {
            Ok(contents) => contents.lines().map(|x| x.to_string()).collect::<Vec<String>>(),
            Err(_) => return Err(())
        };

        let mut playlist = Self {
            songs: contents.into_iter().map(|line| {
                let components = line.split("||").map(|component| component.to_string()).collect::<Vec<String>>();
                Song {
                    name: components[0].clone(),
                    channel: components[1].clone(),
                    url: components[2].clone(),
                    file: match components.get(3) {
                        Some(file) => if *file == String::from("_") { None } else { Some(PathBuf::from(file)) },
                        None => None
                    }
                }
            }).collect::<Vec<Song>>(),
            collected: HashSet::new(),
            file
        };

        let _ = playlist.songs.iter().map(|song| playlist.collected.insert(song.url.clone()));
        Ok(playlist)
    }

    pub fn add_song(&mut self, song: Song) {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.file).unwrap();

        let _ = writeln!(file, "{}", format!("{}||{}||{}||{}", song.name, song.channel, song.url, match song.file {
            Some(ref f) => f.to_string_lossy().to_string(),
            None => String::from("_")
        }));

        self.collected.insert(song.url.clone());
        self.songs.push(song);
    }

    pub fn contains(&self, song: &Song) -> bool {
        return self.collected.contains(&song.url);
    }
}

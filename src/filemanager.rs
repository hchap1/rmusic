use std::collections::HashSet;
use std::fs::read_to_string;
use std::fs::create_dir_all;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::fs::read_dir;
use std::io::Write;
use std::fs::File;

use directories::ProjectDirs;

use crate::downloader::Song;


pub fn get_directory() -> PathBuf {
    let p = ProjectDirs::from("com", "timeparadox", "rmusic").unwrap();
    let path = p.data_dir().to_path_buf();
    let _ = create_dir_all(&path);
    path
}

pub fn find_smallest_unused_id(playlist: &Vec<Song>) -> Result<usize, ()> {
    let mut smallest_id: usize = 0;
    let contents = match read_dir(get_directory()) {
        Ok(contents) => contents,
        Err(_) => return Err(())
    };

    let mut used_ids: HashSet<usize> = HashSet::new();

    for entry in contents {
        let item = match entry {
            Ok(entry) => entry,
            Err(_) => continue
        };

        let filename = item.file_name().to_string_lossy().to_string();
        if let Some(extension) = item.path().extension() {
            let extension = extension.to_string_lossy().to_string();
            if extension == "mp3" {
                let id = match filename.strip_suffix(".mp3").unwrap().parse::<usize>() {
                    Ok(id) => id,
                    Err(_) => continue
                };

                used_ids.insert(id);
            }
        }

        for song in playlist {
            if let Some(f) = &song.file {
                let name = f.file_name().unwrap().to_string_lossy().to_string();
                let id = name.strip_suffix(".mp3").unwrap().parse::<usize>().unwrap();
                used_ids.insert(id);
            }
        }
    }

    while used_ids.contains(&smallest_id) { smallest_id += 1; }
    Ok(smallest_id)
}

pub struct Playlist {
    pub songs: Vec<Song>,
    collected: HashSet<String>,
    file: PathBuf
}

impl Playlist {
    pub fn load_playlist() -> Result<Self, ()> {
        let filepath: String = get_directory().to_string_lossy().to_string() + "/playlist.txt";
        let file: PathBuf = PathBuf::from(filepath);
        let contents = match read_to_string(&file) {
            Ok(contents) => contents.lines().map(|x| x.to_string()).collect::<Vec<String>>(),
            Err(_) => {
                let _ = File::create("playlist.txt");
                vec![]
            }
        };

        let mut playlist = Self {
            songs: contents.into_iter().map(|line| {
                Song::deserialise(line)
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

        let _ = writeln!(file, "{}", song.serialise());

        self.collected.insert(song.url.clone());
        self.songs.push(song);
    }

    pub fn contains(&self, song: &Song) -> bool {
        return self.collected.contains(&song.url);
    }

    pub fn remove_song(&mut self, idx: usize) {
        self.songs.remove(idx);
        
        let mut contents = match read_to_string(&self.file) {
            Ok(contents) => contents.lines().map(|x| x.to_string()).collect::<Vec<String>>(),
            Err(_) => return
        };

        contents.remove(idx);

        let mut file = File::create(&self.file).unwrap();

        for line in contents {
            let _ = writeln!(file, "{line}");
        }
    }
}

use std::fs::read_to_string;
use std::process::Command;
use std::path::PathBuf;
use std::io::Write;
use std::fs::File;

use crate::filemanager::find_smallest_unused_id;
use crate::filemanager::get_directory;

const SEPARATOR: char = '˾';

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Song {
    pub name: String,
    pub channel: String,
    pub url: String,
    pub file: Option<PathBuf>
}

impl Song {
    pub fn serialise(&self) -> String {
        format!("{}{}{}{}{}{}{}", self.name, SEPARATOR, self.channel, SEPARATOR, self.url, SEPARATOR, match self.file {
            Some(ref f) => f.to_string_lossy().to_string(),
            None => String::from("_")
        })
    }

    pub fn deserialise(serial: String) -> Self {
        let components = serial.split(SEPARATOR).map(|component| component.to_string()).collect::<Vec<String>>();
        Self {
            name: components[0].clone(),
            channel: components[1].clone(),
            url: components[2].clone(),
            file: match components.get(3) {
                Some(file) => if *file == String::from("_") { None } else { Some(PathBuf::from(file)) },
                None => None
            }
        }
    }

    pub fn download(&mut self) {
        let file_id: usize = match find_smallest_unused_id() {
            Ok(id) => id,
            Err(_) => return
        };

        let old: String = self.serialise();

        // yt-dlp -f "bestaudio" --extract-audio --audio-format mp3 -o <id>.mp3 <url>

        let _ = Command::new("yt-dlp").arg("-f").arg("bestaudio").arg("--extract-audio").arg("--audio-format").arg("mp3").arg("-o").arg(format!(
            "{}/{}.mp3", get_directory().to_string_lossy().to_string(), file_id
        )).arg(&self.url).spawn();

        self.file = Some(get_directory().join(PathBuf::from(format!("{file_id}.mp3"))));

        let mut contents = match read_to_string(get_directory().join(PathBuf::from("playlist.txt"))) {

            Ok(contents) => contents.lines().map(|x| x.to_string()).collect::<Vec<String>>(),
            Err(_) => return
        };

        let new: String = self.serialise();

        let idx = contents.iter().position(|n| *n == old).unwrap();
        contents[idx] = new;

        let mut file = File::create(get_directory().join(PathBuf::from("playlist.txt"))).unwrap();

        for line in contents {
            let _ = writeln!(file, "{line}");
        }

    }
}

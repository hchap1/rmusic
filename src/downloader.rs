use std::path::PathBuf;

#[derive(PartialEq, Eq, Hash)]
pub struct Song {
    pub name: String,
    pub channel: String,
    pub url: String,
    pub file: Option<PathBuf>
}

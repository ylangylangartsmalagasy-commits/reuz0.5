use std::collections::BTreeMap;
use std::path::PathBuf;
use walkdir::WalkDir;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Track {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Album {
    pub name: String,
    pub tracks: Vec<Track>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Artist {
    pub name: String,
    pub albums: BTreeMap<String, Album>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<Track>,
}

pub fn scan_music_library(root_path: PathBuf) -> BTreeMap<String, Artist> {
    let mut library: BTreeMap<String, Artist> = BTreeMap::new();
    let extensions = ["mp3", "flac", "wav", "ogg", "m4a"];

    for entry in WalkDir::new(root_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if extensions.contains(&ext.to_lowercase().as_str()) {
                    let mut ancestors = path.ancestors();
                    let _file_name = ancestors.next();
                    let album_name = ancestors.next().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or("Album Inconnu").to_string();
                    let artist_name = ancestors.next().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or("Artiste Inconnu").to_string();
                    let track_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Piste Inconnue").to_string();

                    let track = Track { name: track_name, path: path.to_path_buf() };
                    let artist = library.entry(artist_name.clone()).or_insert(Artist { name: artist_name, albums: BTreeMap::new() });
                    let album = artist.albums.entry(album_name.clone()).or_insert(Album { name: album_name, tracks: Vec::new() });
                    album.tracks.push(track);
                }
            }
        }
    }
    library
}
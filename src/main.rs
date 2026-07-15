#![windows_subsystem = "windows"]

use iced::{executor, Application, Command, Element, Length, Settings, Theme, Alignment};
use iced::widget::{button, column, container, row, text, scrollable, text_input, image};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use native_dialog::FileDialog;
use serde::{Serialize, Deserialize};
use std::fs;

mod audio;
mod library;

use audio::AudioEngine;
// IMPORTANT : J'ai ajouté "Album" ici
use library::{scan_music_library, Artist, Album, Playlist, Track};

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackContext {
    Library,
    Playlist(usize),
}

#[derive(Serialize, Deserialize, Clone)]
struct AppConfig {
    last_library_path: Option<PathBuf>,
    current_playlist_idx: Option<usize>,
    playlists: Vec<Playlist>,
}

#[derive(Debug, Clone)]
enum Message {
    SelectLibraryDir,
    AddFolderToLibrary,
    ToggleLibrary,
    PlayTrack(Track, PlaybackContext),
    TogglePlayPause,
    NextTrack,
    PreviousTrack,
    CreatePlaylist,
    DeletePlaylist(usize),
    PlaylistNameChanged(String),
    SelectPlaylist(usize),
    AddTrackToPlaylist(Track, usize),
    RemoveTrackFromPlaylist(usize, usize),
    TogglePlaylist(usize),
}

struct ReuzGUI {
    audio_engine: AudioEngine,
    library: BTreeMap<String, Artist>,
    playlists: Vec<Playlist>,
    selected_playlist_idx: Option<usize>,
    new_playlist_name: String,
    current_track: Option<Track>,
    is_playing: bool,
    library_path: Option<PathBuf>,
    is_library_visible: bool,
    expanded_playlists: HashSet<usize>,
    playback_context: PlaybackContext,
}

impl Application for ReuzGUI {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let config: AppConfig = fs::read_to_string("config.json")
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or(AppConfig { last_library_path: None, current_playlist_idx: None, playlists: vec![Playlist { name: "Favoris".to_string(), tracks: Vec::new() }] });

        let library = config.last_library_path.as_ref().map(|p| scan_music_library(p.clone())).unwrap_or_default();

        (
            Self {
                audio_engine: AudioEngine::new().expect("Échec moteur audio"),
                library,
                playlists: config.playlists,
                selected_playlist_idx: config.current_playlist_idx,
                new_playlist_name: String::new(),
                current_track: None,
                is_playing: false,
                library_path: config.last_library_path,
                is_library_visible: true,
                expanded_playlists: HashSet::new(),
                playback_context: PlaybackContext::Library,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String { String::from("Reuz v.0.5") }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SelectLibraryDir => {
                if let Some(path) = FileDialog::new().show_open_single_dir().unwrap_or(None) {
                    self.library_path = Some(path.clone());
                    self.library = scan_music_library(path);
                }
            }
            Message::AddFolderToLibrary => {
                if let Some(path) = FileDialog::new().show_open_single_dir().unwrap_or(None) {
                    let new_lib = scan_music_library(path);
                    for (artist_name, artist) in new_lib {
                        let entry = self.library.entry(artist_name).or_insert(Artist { name: artist.name, albums: BTreeMap::new() });
                        for (album_name, mut album) in artist.albums {
                            let alb_entry = entry.albums.entry(album_name).or_insert(Album {
                                name: album.name.clone(),
                                tracks: Vec::new(),
                            });
                            alb_entry.tracks.extend(album.tracks);
                        }
                    }
                }
            }
            Message::DeletePlaylist(idx) => {
                self.playlists.remove(idx);
                if self.selected_playlist_idx == Some(idx) { self.selected_playlist_idx = None; }
            }
            Message::ToggleLibrary => self.is_library_visible = !self.is_library_visible,
            Message::PlayTrack(track, context) => {
                self.playback_context = context;
                if let Ok(_) = self.audio_engine.play_file(&track.path) {
                    self.current_track = Some(track);
                    self.is_playing = true;
                }
            }
            Message::TogglePlayPause => {
                self.audio_engine.toggle_pause();
                self.is_playing = !self.audio_engine.is_paused();
            }
            Message::NextTrack => self.change_track(1),
            Message::PreviousTrack => self.change_track(-1),
            Message::PlaylistNameChanged(name) => self.new_playlist_name = name,
            Message::CreatePlaylist => {
                if !self.new_playlist_name.is_empty() {
                    self.playlists.push(Playlist { name: self.new_playlist_name.clone(), tracks: Vec::new() });
                    self.new_playlist_name.clear();
                }
            }
            Message::SelectPlaylist(idx) => self.selected_playlist_idx = Some(idx),
            Message::AddTrackToPlaylist(track, p_idx) => {
                if let Some(p) = self.playlists.get_mut(p_idx) { p.tracks.push(track); }
            }
            Message::RemoveTrackFromPlaylist(p_idx, t_idx) => {
                if let Some(p) = self.playlists.get_mut(p_idx) { p.tracks.remove(t_idx); }
            }
            Message::TogglePlaylist(idx) => {
                if self.expanded_playlists.contains(&idx) { self.expanded_playlists.remove(&idx); }
                else { self.expanded_playlists.insert(idx); }
            }
        }
        self.save_config();
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let left_column = if self.is_library_visible {
            let mut content = column![row![
    button(image("assets/closefolder.png").width(20).height(20))
        .on_press(Message::ToggleLibrary)
        .style(iced::theme::Button::Text), // <--- AJOUTEZ CECI
    text(" Bibliothèque"),
    button(text("+"))
        .on_press(Message::AddFolderToLibrary)
        .style(iced::theme::Button::Text) // <--- AJOUTEZ CECI
].spacing(10)].spacing(10);

            for artist in self.library.values() {
                for album in artist.albums.values() {
                    for track in &album.tracks {
                        let mut row_content = row![].spacing(5);
                        if let Some(p_idx) = self.selected_playlist_idx {
                            row_content = row_content.push(button(image("assets/plus.png").width(20).height(20)).on_press(Message::AddTrackToPlaylist(track.clone(), p_idx)).style(iced::theme::Button::Text));
                        }
                        row_content = row_content.push(button(text(&track.name).size(13)).on_press(Message::PlayTrack(track.clone(), PlaybackContext::Library)).style(iced::theme::Button::Text));
                        content = content.push(row_content);
                    }
                }
            }
            container(scrollable(content)).width(Length::FillPortion(1))
        } else {
            container(
                button(image("assets/openfolden.png").width(20).height(20))
                    .on_press(Message::ToggleLibrary)
                    .style(iced::theme::Button::Text) // <--- AJOUTEZ CECI
            ).width(Length::FillPortion(1))
        };

        let center_column = container(column![
            image("assets/reuz.jpg").width(150.0).height(150.0),
            text(self.current_track.as_ref().map(|t| t.name.clone()).unwrap_or("Aucune piste".to_string())).size(20)
        ].align_items(Alignment::Center).spacing(20)).width(Length::FillPortion(3)).center_x().center_y();

        let mut playlist_list = column![];
        for (i, p) in self.playlists.iter().enumerate() {
            let is_expanded = self.expanded_playlists.contains(&i);
            let toggle_icon = if is_expanded { "assets/close.png" } else { "assets/openlist.png" };

            let mut p_item = column![row![
                button(image(toggle_icon).width(20).height(20)).on_press(Message::TogglePlaylist(i)).style(iced::theme::Button::Text),
                button(text(&p.name)).on_press(Message::SelectPlaylist(i)).style(iced::theme::Button::Text),
                button(text("X"))
        .on_press(Message::DeletePlaylist(i))
        .style(iced::theme::Button::Text)
            ]];

            if is_expanded {
                for (t_idx, t) in p.tracks.iter().enumerate() {
                    p_item = p_item.push(row![
                        button(image("assets/close.png").width(15).height(15)).on_press(Message::RemoveTrackFromPlaylist(i, t_idx)).style(iced::theme::Button::Text),
                        button(text(&t.name).size(12))
                            .on_press(Message::PlayTrack(t.clone(), PlaybackContext::Playlist(i)))
                            .style(iced::theme::Button::Text)
                    ]);
                }
            }
            playlist_list = playlist_list.push(p_item);
        }

        let right_column = container(column![
            text("Playlists"),
            row![text_input("...", &self.new_playlist_name).on_input(Message::PlaylistNameChanged), button(image("assets/plus.png").width(20).height(20)).on_press(Message::CreatePlaylist).style(iced::theme::Button::Text)],
            scrollable(playlist_list)
        ]).width(Length::FillPortion(1));

        let controls = row![
            button(image("assets/previous.png").width(30).height(30)).on_press(Message::PreviousTrack).style(iced::theme::Button::Text),
            button(image(if self.is_playing { "assets/pause.png" } else { "assets/play.png" }).width(30).height(30)).on_press(Message::TogglePlayPause).style(iced::theme::Button::Text),
            button(image("assets/next.png").width(30).height(30)).on_press(Message::NextTrack).style(iced::theme::Button::Text),
        ].spacing(20).align_items(Alignment::Center);

        container(column![
            row![left_column, center_column, right_column].height(Length::Fill),
            container(controls).width(Length::Fill).center_x().padding(10)
        ]).padding(10).style(iced::theme::Container::Custom(Box::new(CustomBackground))).into()
    }

    fn theme(&self) -> Theme { Theme::Dark }
}

impl ReuzGUI {
    fn change_track(&mut self, direction: i32) {
        let tracks = match self.playback_context {
            PlaybackContext::Library => {
                let mut all = Vec::new();
                for artist in self.library.values() {
                    for album in artist.albums.values() {
                        all.extend(album.tracks.clone());
                    }
                }
                all
            }
            PlaybackContext::Playlist(idx) => {
                self.playlists.get(idx).map(|p| p.tracks.clone()).unwrap_or_default()
            }
        };

        if tracks.is_empty() { return; }

        if let Some(current) = &self.current_track {
            if let Some(pos) = tracks.iter().position(|t| t.path == current.path) {
                let len = tracks.len() as i32;
                let new_idx = ((pos as i32 + direction).rem_euclid(len)) as usize;
                let next_track = tracks[new_idx].clone();
                let _ = self.audio_engine.play_file(&next_track.path);
                self.current_track = Some(next_track);
                self.is_playing = true;
            }
        }
    }

    fn save_config(&self) {
        let config = AppConfig { last_library_path: self.library_path.clone(), current_playlist_idx: self.selected_playlist_idx, playlists: self.playlists.clone() };
        let _ = fs::write("config.json", serde_json::to_string(&config).unwrap());
    }
}

struct CustomBackground;
impl container::StyleSheet for CustomBackground {
    type Style = Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance { background: Some(iced::Background::Color(iced::Color::from_rgb(0.0, 0.0, 0.0))), ..Default::default() }
    }
}

fn main() -> iced::Result {
    let icon = iced::window::icon::from_file_data(include_bytes!("../assets/reuz.ico"), None).ok();

    ReuzGUI::run(Settings {
        window: iced::window::Settings {
            icon, // On applique l'icône ici
            ..Default::default()
        },
        ..Settings::default()
    })
}
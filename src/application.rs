use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::prelude::Stylize;
use ratatui::widgets::ListState;
use ratatui::{
    symbols::border,
    text::Line,
    widgets::{
        Block,
        List
    },
    DefaultTerminal,
    Frame,
    style::Style,
    layout::{
        Layout,
        Constraint
    }
};
use rand::seq::SliceRandom;
use rand::rng;

use crate::filemanager::find_smallest_unused_id;
use crate::chromedriver::search_youtube;
use crate::filemanager::Playlist;
use crate::audio::AudioPlayer;
use crate::downloader::Song;

#[derive(PartialEq, Eq)]
pub enum ApplicationState {
    Homepage,
    Search,
    Playlist
}

#[derive(PartialEq, Eq)]
enum Mode {
    Normal,
    Input
}

pub struct Application {
    state: ApplicationState,
    mode: Mode,
    list_state: ListState,

    user_input: Vec<char>,
    playlist: Playlist,
    search_results: Vec<Song>,

    running: bool,
    audio_player: AudioPlayer
}

impl Application {
    pub fn new() -> Self {
        Self {
            state: ApplicationState::Homepage,
            mode: Mode::Normal,
            list_state: ListState::default(),
            user_input: Vec::new(),
            playlist: Playlist::load_playlist().unwrap(),
            search_results: Vec::new(),
            running: true,
            audio_player: AudioPlayer::new()
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) {
        self.list_state.select(Some(0));
        while self.running {
            let _ = terminal.draw(|frame| self.draw(frame));
            let event = match event::read() {
                Ok(e) => e,
                Err(_) => {
                    self.running = false;
                    return;
                }
            };

            match event {
                Event::Key(k) => self.handle_input(k).await,
                _ => {}
            }
        }
    }

    async fn handle_input(&mut self, k: KeyEvent) {
        match k.code {
            KeyCode::Char(c) => {
                match self.state {
                    ApplicationState::Search => {
                        if self.mode == Mode::Input {
                            self.user_input.push(c);
                        } else {
                            match c {
                                'i' => self.mode = Mode::Input,
                                'q' => self.running = false,
                                'j' => self.list_state.select_next(),
                                'k' => self.list_state.select_previous(),
                                ' ' => self.audio_player.toggle(),
                                 _  => {}
                            }
                        }
                    }

                    ApplicationState::Homepage => {
                        match c {
                            'j' => self.list_state.select_next(),
                            'k' => self.list_state.select_previous(),
                            'q' => self.running = false,
                            ' ' => self.audio_player.toggle(),
                            _ => {}
                        }
                    }

                    ApplicationState::Playlist => {
                        match c {
                            'j' => self.list_state.select_next(),
                            'k' => self.list_state.select_previous(),
                            'q' => self.running = false,
                            ' ' => self.audio_player.toggle(),
                            'a' => {
                                let idx = match self.list_state.selected() {
                                    Some(idx) => if idx < self.playlist.songs.len() { idx } else { return; },
                                    None => return
                                };
                                self.audio_player.append(self.playlist.songs[idx].clone());
                            }
                            _ => {}
                        }
                    }
                }
            }

            KeyCode::Backspace => {
                match self.state {
                    ApplicationState::Homepage => {},
                    ApplicationState::Search => {
                        if self.mode == Mode::Input {
                            self.user_input.pop();
                        } else {
                            self.state = ApplicationState::Homepage;
                        }
                    }
                    ApplicationState::Playlist => {
                        if self.mode == Mode::Input {
                            self.user_input.pop();
                        } else {
                            self.state = ApplicationState::Homepage;
                        }
                    }
                }
            }

            KeyCode::Delete => {
                if self.state == ApplicationState::Playlist {
                    match self.list_state.selected() {
                        Some(idx) => if idx < self.playlist.songs.len() { self.playlist.remove_song(idx) },
                        None => {}
                    }
                }
            }

            KeyCode::Enter => {
                match self.state {
                    ApplicationState::Homepage => {
                        let idx = match self.list_state.selected() {
                            Some(n) => n,
                            None => 0
                        };

                        if idx == 0 {
                            self.state = ApplicationState::Search;
                            self.list_state.select(Some(0));
                        } else if idx == 1 {
                            self.state = ApplicationState::Playlist;
                            self.list_state.select(Some(0));
                        } else {
                            self.shuffle();
                        }
                    }

                    ApplicationState::Playlist => {
                        if self.mode == Mode::Normal {
                            let idx = match self.list_state.selected() {
                                Some(idx) => idx,
                                None => return
                            };

                            if idx < self.playlist.songs.len() {
                                if self.playlist.songs[idx].file == None {
                                    let i = find_smallest_unused_id(&self.playlist.songs).unwrap();
                                    self.playlist.songs[idx].download(i);
                                }
                                else {
                                    self.audio_player.play(self.playlist.songs[idx].clone());
                                }
                            }
                        }
                    }

                    ApplicationState::Search => {
                        if self.mode == Mode::Input {
                            let _ = self.mode == Mode::Normal;
                            self.fill_search_criteria().await;
                        } else {
                            self.select_search_option();
                        }
                    }
                }
            }

            KeyCode::Esc => { self.mode = Mode::Normal; self.user_input.clear(); },

            KeyCode::Right => self.audio_player.skip(),

            _ => {}
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let layout = Layout::default().direction(ratatui::layout::Direction::Horizontal)
            .constraints(vec![
                    Constraint::Percentage(70),
                    Constraint::Percentage(30)
            ]).split(frame.area());
        let block: Block = Block::bordered().border_set(border::ROUNDED).title_top(Line::from(match self.state {
            ApplicationState::Search => "[ BROWSE SONGS ]",
            ApplicationState::Homepage => "[ HOMEPAGE ]",
            ApplicationState::Playlist => "[ SONGS ]",
        }).centered().light_blue()).title_bottom(Line::from(format!("[ {} ]", self.user_input.iter().collect::<String>())).centered().white());

        let lines: List = List::new(
            match self.state {
                ApplicationState::Homepage => vec!["Browse Songs", "View Playlist", "Shuffle Play"].into_iter().map(|x| Line::from(x)).collect::<Vec<Line>>(),
                ApplicationState::Search => self.search_results.iter().map(|song| {
                    let line = Line::from(song.name.clone());
                    match self.playlist.contains(song) {
                        true => line.green(),
                        false => line.white()
                    }
                }).collect::<Vec<Line>>(),
                ApplicationState::Playlist => self.playlist.songs.iter().map(|song| {
                    let line = Line::from(song.name.clone());
                    match song.file {
                        Some(_) => line.white(),
                        None => line.gray()
                    }
                }).collect::<Vec<Line>>()
            }
        ).block(block).highlight_style(Style::new()).highlight_symbol("->");

        let queue: List = List::new(
            self.audio_player.get_queue().iter().enumerate().map(|x| Line::from(
                if x.0 == 0 { x.1.clone().green() } else { x.1.clone().white() }
            )).collect::<Vec<Line>>()
        ).block(
                Block::bordered().border_set(border::ROUNDED).title("QUEUE".light_blue().into_centered_line())
            );

        frame.render_stateful_widget(lines, layout[0], &mut self.list_state);
        frame.render_widget(queue, layout[1]);
    }

    async fn fill_search_criteria(&mut self) {
        self.search_results = search_youtube(self.user_input.iter().collect::<String>()).await.unwrap();
    }

    fn select_search_option(&mut self) {
        if self.state != ApplicationState::Search {
            return;
        }

        let idx = match self.list_state.selected() {
            Some(idx) => if idx < self.search_results.len() { idx } else { return; },
            None => return
        };

        if !self.playlist.contains(&self.search_results[idx]) { self.playlist.add_song(self.search_results[idx].clone()); }
    }

    fn shuffle(&mut self) {
        let mut rng = rng();
        let mut songs: Vec<Song> = self.playlist.songs.iter().filter(|x| x.file != None).map(|x| x.clone()).collect();
        songs.shuffle(&mut rng);
        for song in &songs {
            self.audio_player.append(song.clone());
        }
    }
}

use std::time::Duration;

#[derive(Clone, Default, Debug)]
pub struct Song {
    pub name: String,
    pub path: String,
}

impl Song {
    pub fn from(name: String, path: String) -> Self {
        Song { name, path }
    }
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum SongState {
    #[default]
    NONE,

    PLAY,
    PAUSE,
    STOP, // stop playing, a.k.a. end of play
}

#[derive(Clone, Default, Debug)]
pub struct ActiveSong {
    pub song: Option<Song>,
    pub state: SongState,
    pub progress: Duration,
    pub duration: Duration,
}

impl ActiveSong {
    pub fn from(song: Song, duration: Duration) -> ActiveSong {
        ActiveSong {
            song: Some(song),
            state: SongState::NONE,
            progress: Duration::from_secs(0),
            duration,
        }
    }
}

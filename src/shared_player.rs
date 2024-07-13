use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, RwLock},
    thread::{spawn, JoinHandle},
};

use rodio::{Decoder, OutputStream, Sink, Source};

use crate::{
    asset::{PlaybackMode, PlayerAsset},
    make::Make,
    player::Player,
    song::{ActiveSong, Song, SongState},
};

pub type SharedPlayer = Arc<RwLock<PlayerAsset>>;

impl Make<Self> for SharedPlayer {
    fn make() -> SharedPlayer {
        Arc::new(RwLock::new(PlayerAsset::make()))
    }
}

impl Player for SharedPlayer {
    fn add(&self, song: Song) {
        self.write().unwrap().waiting_q.push(song);
    }

    fn waiting_list(&self) -> Vec<Song> {
        self.read()
            .unwrap()
            .waiting_q
            .iter()
            .map(Clone::clone)
            .collect()
    }

    fn played_list(&self) -> Vec<Song> {
        self.read()
            .unwrap()
            .played_q
            .iter()
            .map(Clone::clone)
            .collect()
    }

    fn current_song(&self) -> ActiveSong {
        self.read().unwrap().current.clone()
    }

    fn play(&self) -> JoinHandle<()> {
        if self.is_playing() {
            return spawn(|| {});
        };
        // acquire an arc for child thread
        let state = Arc::clone(&self);
        // create a new thread for loading and playing music
        spawn(move || {
            // The life cycle of "_stream" should >= source
            // so we should make a new sink each time before playing some source
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            {
                // acquire write lock to place a new sink
                let mut state = state.write().unwrap();
                state.sink = Some(Sink::try_new(&stream_handle).unwrap());
            }
            loop {
                let song = { state.write().unwrap().waiting_q.pop() };
                if song.is_none() {
                    break;
                }
                let song = song.unwrap();
                let file = BufReader::new(File::open(song.path.clone()).unwrap());
                let source = Decoder::new(file).unwrap();
                {
                    // acquire write lock to prepare playing song
                    let mut state = state.write().unwrap();
                    state.current =
                        ActiveSong::from(song.clone(), source.total_duration().unwrap_or_default());
                    state.current.state = SongState::PLAY;
                }
                {
                    // acquire read lock to play music
                    let state = state.read().unwrap();
                    // acquiring a read lock to play the music
                    if let Some(sink) = state.sink.as_ref() {
                        // assign current song
                        sink.append(source);
                        sink.set_volume(state.volume);
                        sink.sleep_until_end();
                    };
                }
                {
                    // acquire write lock to finish end-of-play process
                    let mut state = state.write().unwrap();
                    state.current.progress = state.current.duration;
                    state.current.state = SongState::STOP;
                    state.current.song = None;
                    state.played_q.push(song.clone());
                }
                {
                    // auto play if flag is on, otherwise breaks
                    let to_auto_play = { state.read().unwrap().mode == PlaybackMode::AUTO };
                    if !to_auto_play {
                        break;
                    }
                }
            }
        })
    }

    fn toggle(&self) {
        // acquire an arc for this thread
        let state = Arc::clone(&self);
        // check if old sink exists and
        // play/pause it by acquiring read lock
        if let Some(sink) = &state.read().unwrap().sink {
            if sink.is_paused() {
                sink.play();
            } else {
                sink.pause();
            }
        };
    }

    fn stop(&self) {
        // acquire an arc for this thread
        let state = Arc::clone(&self);
        // check if old sink exists and
        // stop it by acquiring read lock
        if let Some(sink) = &state.read().unwrap().sink {
            sink.stop();
        };

        self.clear(); // clear the whole list
    }

    /// Clear the playlist
    fn clear(&self) {
        // acquire an arc for this thread
        let state = Arc::clone(&self);
        let mut state = state.write().unwrap();
        state.waiting_q.clear();
        state.played_q.clear();
    }

    fn is_playing(&self) -> bool {
        // acquire an arc for this thread
        let state = Arc::clone(&self);
        let res = state.read().unwrap().current.state == SongState::PLAY;
        res
    }

    fn use_normal_play(&self) {
        let state = Arc::clone(&self);
        state.write().unwrap().mode = PlaybackMode::NORMAL;
    }

    fn use_auto_play(&self) {
        let state = Arc::clone(&self);
        state.write().unwrap().mode = PlaybackMode::AUTO;
    }
}

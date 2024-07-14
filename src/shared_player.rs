use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, RwLock},
    thread::{spawn, JoinHandle},
};

use rodio::{Decoder, Sink, Source};

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
    fn add(&self, song: Song) -> JoinHandle<()> {
        // acquire an arc for this thread
        let state = Arc::clone(&self);
        spawn(move || {
            state.write().unwrap().waiting_q.push(song);
        })
    }

    fn waiting_list(&self) -> JoinHandle<Vec<Song>> {
        // acquire an arc for this thread
        let state = Arc::clone(&self);
        spawn(move || {
            state
                .read()
                .unwrap()
                .waiting_q
                .iter()
                .map(Clone::clone)
                .collect()
        })
    }

    fn played_list(&self) -> JoinHandle<Vec<Song>> {
        // acquire an arc for this thread
        let state = Arc::clone(&self);
        spawn(move || {
            state
                .read()
                .unwrap()
                .played_q
                .iter()
                .map(Clone::clone)
                .collect()
        })
    }

    fn current_song(&self) -> JoinHandle<ActiveSong> {
        // acquire an arc for this thread
        let state = Arc::clone(&self);
        spawn(move || state.read().unwrap().current.clone())
    }

    fn play(&self) -> JoinHandle<()> {
        if self.is_playing().join().unwrap() {
            return spawn(|| {});
        };
        // acquire an arc for child thread
        let state = Arc::clone(&self);
        // create a new thread for loading and playing music
        spawn(move || {
            // The life cycle of "_stream" should >= source
            // so we should make a new sink each time before playing some source
            let (_stream, stream_handle) = { (state.read().unwrap().gen_out)() };
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

    fn toggle(&self) -> JoinHandle<()> {
        // acquire an arc for this thread
        let state = Arc::clone(&self);
        spawn(move || {
            // check if old sink exists and
            // play/pause it by acquiring read lock
            if let Some(sink) = &state.read().unwrap().sink {
                if sink.is_paused() {
                    sink.play();
                } else {
                    sink.pause();
                }
            };
        })
    }

    fn stop(&self) -> JoinHandle<()> {
        // acquire an arc for this thread
        let state = Arc::clone(&self);
        spawn(move || {
            // check if old sink exists and
            // stop it by acquiring read lock
            if let Some(sink) = &state.read().unwrap().sink {
                sink.stop();
            };
        })
    }

    /// Clear the playlist
    fn clear(&self) -> JoinHandle<()> {
        // acquire an arc for this thread
        let state = Arc::clone(&self);
        spawn(move || {
            let mut state = state.write().unwrap();
            state.waiting_q.clear();
            state.played_q.clear();
        })
    }

    fn is_playing(&self) -> JoinHandle<bool> {
        let state = Arc::clone(&self);
        spawn(move || {
            // acquire an arc for this thread
            let res = state.read().unwrap().current.state == SongState::PLAY;
            res
        })
    }

    fn use_normal_play(&self) -> JoinHandle<()> {
        let state = Arc::clone(&self);
        spawn(move || {
            state.write().unwrap().mode = PlaybackMode::NORMAL;
        })
    }

    fn use_auto_play(&self) -> JoinHandle<()> {
        let state = Arc::clone(&self);
        spawn(move || {
            state.write().unwrap().mode = PlaybackMode::AUTO;
        })
    }

    /// Set output device generator, the default
    /// generator is based on `OutputStream::try_default`.
    ///
    /// ```
    /// use crate::super_rodio::{Make, SharedPlayer, Player};
    /// use rodio::OutputStream;
    ///
    /// let player = SharedPlayer::make();
    /// player.set_device_maker(Box::new(move || {
    ///     OutputStream::try_default().unwrap()
    /// }));
    /// ```
    fn set_device_maker(
        &self,
        with_generator: Box<
            dyn Fn() -> (rodio::OutputStream, rodio::OutputStreamHandle) + Send + Sync,
        >,
    ) -> JoinHandle<()> {
        let state = Arc::clone(&self);
        spawn(move || {
            state.write().unwrap().gen_out = with_generator;
        })
    }
}

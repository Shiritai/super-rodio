use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, RwLock},
    thread::{spawn, JoinHandle},
    time::Duration,
};

use limited_queue::LimitedQueue;
use rodio::{Decoder, OutputStream, Sink, Source};

/// Trait that this structure can
/// be made without argument
pub trait Make<T> {
    /// Make a new structure
    /// 
    /// Just like the well-known `new` method
    /// with different name to avoid name collision
    /// after rust reference coercion
    fn make() -> T;
}

#[derive(Clone, Default, Debug)]
pub struct Song {
    name: String,
    path: String,
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum PlaybackState {
    #[default]
    NONE,

    PLAY,
    PAUSE,
    STOP, // stop playing, a.k.a. end of play
}

#[derive(Clone, Default, Debug)]
pub struct ActiveSong {
    song: Song,
    state: PlaybackState,
    progress: Duration,
    duration: Duration,
}

impl ActiveSong {
    pub fn from(song: Song, duration: Duration) -> ActiveSong {
        ActiveSong {
            song,
            state: PlaybackState::NONE,
            progress: Duration::from_secs(0),
            duration,
        }
    }
}

pub struct PlayerState {
    sink: Option<Sink>,
    waiting_q: LimitedQueue<Song>, // waiting queue
    current: ActiveSong,
    played_q: LimitedQueue<Song>, // played queue
    volume: f32,
}

impl Make<Self> for PlayerState {
    fn make() -> PlayerState {
        PlayerState {
            sink: None,
            waiting_q: LimitedQueue::with_capacity(1000),
            current: Default::default(),
            played_q: LimitedQueue::with_capacity(1000),
            volume: 0.5f32,
        }
    }
}

pub type SharedPlayer = Arc<RwLock<PlayerState>>;

impl Make<Self> for SharedPlayer {
    fn make() -> SharedPlayer {
        Arc::new(RwLock::new(PlayerState::make()))
    }
}

pub trait Player {
    fn add(&self, song: Song);
    fn play(&self) -> JoinHandle<()>;
    fn toggle(&self);
    fn stop(&self);
    fn is_playing(&self) -> bool;
}

impl Player for SharedPlayer {
    fn add(&self, song: Song) {
        self.write().unwrap().waiting_q.push(song);
    }

    fn play(&self) -> JoinHandle<()> {
        if self.is_playing() {
            return spawn(|| {});
        };
        // acquire an arc for child thread
        let state = Arc::clone(&self);
        let song = { self.write().unwrap().waiting_q.pop() };
        match song {
            None => spawn(|| {}),
            Some(song) => {
                // create a new thread for loading and playing music
                spawn(move || {
                    // The life cycle of "_stream" should >= source
                    // so we should make a new sink each time before playing some source
                    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                    let file = BufReader::new(File::open(song.path.clone()).unwrap());
                    let source = Decoder::new(file).unwrap();
                    {
                        // acquire write lock to place a new sink
                        let mut state = state.write().unwrap();
                        state.sink = Some(Sink::try_new(&stream_handle).unwrap());
                        state.current = ActiveSong::from(song.clone(), source.total_duration().unwrap_or_default());
                        state.current.state = PlaybackState::PLAY;
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
                        state.current.state = PlaybackState::STOP;
                        state.played_q.push(song);
                    }
                })
            }
        }
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
    }

    fn is_playing(&self) -> bool {
        // acquire an arc for this thread
        let state = Arc::clone(&self);
        let res = state.read().unwrap().current.state == PlaybackState::PLAY;
        res
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use crate::{Make, Player, SharedPlayer, Song};

    #[test]
    fn test_play_stop() {
        let player = SharedPlayer::make();
        player.add(Song {
            name: "Music".into(),
            path: "audio/music".into(),
        });

        let t = player.play();
        sleep(Duration::from_secs(5));
        player.stop();
        let _ = t.join();
    }

    #[test]
    fn test_play_resume_stop() {
        let player = SharedPlayer::make();
        player.add(Song {
            name: "Music".into(),
            path: "audio/music".into(),
        });

        let t = player.play();
        sleep(Duration::from_secs(5));
        player.toggle();
        sleep(Duration::from_secs(3));
        player.toggle();
        sleep(Duration::from_secs(5));
        player.stop();
        let _ = t.join();
    }

    #[test]
    fn test_play_songs() {
        let player = SharedPlayer::make();
        player.add(Song {
            name: "Music".into(),
            path: "audio/music".into(),
        });
        
        player.add(Song {
            name: "ShortSound".into(),
            path: "audio/short_sound".into(),
        });
        
        player.add(Song {
            name: "Music".into(),
            path: "audio/music".into(),
        });

        // first song
        let _ = player.play();
        sleep(Duration::from_secs(5));
        
        sleep(Duration::from_secs(5));
        player.stop();

        // second song (short)
        let t = player.play();
        let _ = t.join();
        // third song
        let _ = player.play();
        sleep(Duration::from_secs(5));
        sleep(Duration::from_secs(5));
        player.stop();
    }
}

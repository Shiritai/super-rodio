use limited_queue::LimitedQueue;
use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::{
    make::Make,
    song::{ActiveSong, Song},
};

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum PlaybackMode {
    #[default]
    NORMAL,

    /// Auto play the audio in waiting queue
    AUTO,
}

pub struct PlayerAsset {
    pub sink: Option<Sink>,
    pub waiting_q: LimitedQueue<Song>, // waiting queue
    pub current: ActiveSong,
    pub played_q: LimitedQueue<Song>, // played queue
    pub volume: f32,
    pub mode: PlaybackMode,
    pub gen_out: Box<dyn Fn() -> (OutputStream, OutputStreamHandle) + Send + Sync>,
}

impl Make<Self> for PlayerAsset {
    fn make() -> PlayerAsset {
        PlayerAsset {
            sink: None,
            waiting_q: LimitedQueue::with_capacity(1000),
            current: Default::default(),
            played_q: LimitedQueue::with_capacity(1000),
            volume: 0.5f32,
            mode: Default::default(),
            gen_out: Box::new(|| OutputStream::try_default().unwrap()),
        }
    }
}

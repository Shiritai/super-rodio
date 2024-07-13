use std::thread::JoinHandle;

use rodio::{OutputStream, OutputStreamHandle};

use crate::song::{ActiveSong, Song};

pub trait Player {
    fn add(&self, song: Song);
    fn waiting_list(&self) -> Vec<Song>;
    fn played_list(&self) -> Vec<Song>;
    fn current_song(&self) -> ActiveSong;
    fn play(&self) -> JoinHandle<()>;
    fn use_normal_play(&self);
    fn use_auto_play(&self);
    fn toggle(&self);
    fn stop(&self);
    fn clear(&self);
    fn is_playing(&self) -> bool;
    fn set_device_maker(
        &self,
        with_generator: Box<dyn Fn() -> (OutputStream, OutputStreamHandle) + Send + Sync>,
    );
}

use std::thread::JoinHandle;

use rodio::{OutputStream, OutputStreamHandle};

use crate::song::{ActiveSong, Song};

pub trait Player {
    /// Add a song to the player
    fn add(&self, song: Song);
    /// Get current waiting list
    fn waiting_list(&self) -> Vec<Song>;
    /// Get current played history
    fn played_list(&self) -> Vec<Song>;
    /// Get current active song
    fn current_song(&self) -> ActiveSong;
    /// Play the song in waiting list
    fn play(&self) -> JoinHandle<()>;
    /// Use normal play mode: playing a single song and stop
    fn use_normal_play(&self);
    /// Use auto play mode: playing all the songs one-by-one in the playlist
    fn use_auto_play(&self);
    /// Toggle play/pause
    fn toggle(&self);
    /// Stop current music and clear all songs in waiting/played list
    fn stop(&self);
    /// Clear all songs in waiting/played list
    fn clear(&self);
    /// Check whether the current song is playing
    fn is_playing(&self) -> bool;
    /// Set output device generator, the default
    /// generator is based on `OutputStream::try_default`
    fn set_device_maker(
        &self,
        with_generator: Box<dyn Fn() -> (OutputStream, OutputStreamHandle) + Send + Sync>,
    );
}

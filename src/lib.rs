mod asset;
mod make;
mod player;
mod shared_player;
mod song;

pub use make::Make;
pub use player::Player;
pub use shared_player::SharedPlayer;
pub use song::Song;

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use rodio::{cpal::{self, traits::HostTrait}, DeviceTrait, OutputStream};

    use crate::{Make, Player, SharedPlayer, Song};

    #[test]
    fn test_play_stop() {
        let player = SharedPlayer::make();
        player.add(Song::from("Music".into(), "audio/music".into()));

        let t = player.play();
        sleep(Duration::from_secs(5));
        player.stop();
        let _ = t.join();
    }

    #[test]
    fn test_play_resume_stop() {
        let player = SharedPlayer::make();
        player.add(Song::from("Music".into(), "audio/music".into()));

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
        player.add(Song::from("Music".into(), "audio/music".into()));

        // first song
        let _ = player.play();
        sleep(Duration::from_secs(2));
        println!("{:?}", player.current_song());

        sleep(Duration::from_secs(2));
        player.stop();
        println!("{:?}", player.current_song());

        player.add(Song::from("ShortSound".into(), "audio/short_sound".into()));

        // second song (short)
        let t = player.play();
        println!("{:?}", player.current_song());
        let _ = t.join();

        player.add(Song::from("Music".into(), "audio/music".into()));
        // third song
        let _ = player.play();
        sleep(Duration::from_secs(2));
        println!("{:?}", player.current_song());
        sleep(Duration::from_secs(2));
        player.stop();
    }

    #[test]
    fn test_auto_play() {
        let player = SharedPlayer::make();
        player.add(Song::from("Music".into(), "audio/short_sound".into()));
        player.add(Song::from("Music".into(), "audio/short_sound".into()));
        player.add(Song::from("Music".into(), "audio/short_sound".into()));

        player.use_auto_play();
        let _ = player.play().join();
    }

    #[test]
    fn test_normal_auto_play() {
        let player = SharedPlayer::make();
        player.add(Song::from("Music".into(), "audio/short_sound".into()));
        player.add(Song::from("Music".into(), "audio/short_sound".into()));
        player.add(Song::from("Music".into(), "audio/short_sound".into()));

        player.use_auto_play();
        let _ = player.play().join();

        sleep(Duration::from_secs(2)); // wait a while...

        player.add(Song::from("Music".into(), "audio/short_sound".into()));
        player.add(Song::from("Music".into(), "audio/short_sound".into()));
        player.add(Song::from("Music".into(), "audio/short_sound".into()));

        player.use_normal_play();
        let _ = player.play().join();
        sleep(Duration::from_secs_f32(0.5)); // wait a while...
        let _ = player.play().join();
        sleep(Duration::from_secs_f32(0.5)); // wait a while...
        let _ = player.play().join();
    }

    #[test]
    fn test_stop() {
        let player = SharedPlayer::make();
        for _ in 0..10 {
            player.add(Song::from("Music".into(), "audio/short_sound".into()));
        }
        player.use_auto_play();

        let t = player.play();
        sleep(Duration::from_secs(3));
        player.stop();
        let _ = t.join(); // should stop immediately

        sleep(Duration::from_secs(1));
        for _ in 0..10 {
            player.add(Song::from("Music".into(), "audio/short_sound".into()));
        }

        let t = player.play();
        sleep(Duration::from_secs(3));
        let _ = t.join(); // should stop immediately
    }

    #[test]
    fn test_choose_output_device() {
        // let (_s, h) = OutputStream::try_default().unwrap();
        for host_id in cpal::available_hosts() {
            println!("In host: {:?}", host_id);
            let host = cpal::host_from_id(host_id).unwrap();
            println!("\tInput devices:");
            for in_d in host.input_devices().unwrap() {
                let in_d = in_d.name().unwrap();
                println!("\t\t{}", in_d);
            }
            println!("\tOutput devices:");
            for out_d in host.output_devices().unwrap() {
                let out_d = out_d.name().unwrap();
                println!("\t\t{}", out_d);
            }
        }
    }
}

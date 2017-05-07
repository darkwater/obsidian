extern crate time;
extern crate alsa;

use components::*;
use status::StatusItem;
use std::sync::mpsc;
use std::thread;

pub struct VolumeStatusItem;
impl StatusItem for VolumeStatusItem {
    fn check_available(&self) -> bool {
        let mixer = alsa::mixer::Mixer::new("hw:3", true).expect("Mixer not found");
        let sid = alsa::mixer::SelemId::new("Speaker", 0);
        let selem = mixer.find_selem(&sid).expect("Control not found");
        let has_volume = selem.has_volume();

        has_volume
    }

    fn get_update_fun(&self) -> fn(mpsc::Sender<Vec<StatusChange>>) {
        fn fun(sx: mpsc::Sender<Vec<StatusChange>>) {
            let changes = vec![
                StatusChange::Text("100%".to_string()),
                StatusChange::Size(SizeRequest::Set)
            ];

            let _ = sx.send(changes);

            loop {
                let mixer = alsa::mixer::Mixer::new("hw:3", true).expect("Mixer not found");
                let sid = alsa::mixer::SelemId::new("Speaker", 0);
                let selem = mixer.find_selem(&sid).expect("Control not found");
                let volume_range = selem.get_playback_volume_range();
                let volume = selem.get_playback_volume(alsa::mixer::SelemChannelId::FrontLeft).expect("Control has no volume range");
                let volume_percent = (volume - volume_range.0) * 100 / (volume_range.1 - volume_range.0);

                let text = format!("{:.0}%", volume_percent);

                let color = (0.2, 1.0, 0.5, 0.95);

                let changes = vec![
                    StatusChange::Text(text),
                    StatusChange::Color(color)
                ];

                let _ = sx.send(changes);

                let sleep_time = ::std::time::Duration::from_secs(5);
                thread::sleep(sleep_time);
            }
        }

        fun
    }
}

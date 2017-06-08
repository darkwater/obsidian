extern crate time;
extern crate alsa;

use components::*;
use config::Config;
use status::StatusItem;
use std::sync::mpsc;
use std::thread;

pub struct VolumeStatusItem;
impl StatusItem for VolumeStatusItem {
    fn check_available(&self) -> Result<(), &str> {
        let mixer = alsa::mixer::Mixer::new("hw:3", true).map_err(|_| "Mixer not found")?;
        let sid = alsa::mixer::SelemId::new("Speaker", 0);
        let selem = mixer.find_selem(&sid).ok_or("Control not found")?;
        let has_volume = selem.has_volume();

        Ok(())
    }

    fn get_update_fun(&self) -> fn(mpsc::Sender<Vec<StatusChange>>, &'static Config) {
        fn fun(sx: mpsc::Sender<Vec<StatusChange>>, config: &'static Config) {
            let changes = vec![
                StatusChange::Icon("volume_up".to_string()),
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

                let color = match volume_percent {
                    0...20 => config.get_color("blue"),
                   21...40 => config.get_color("green"),
                   41...85 => config.get_color("yellow"),
                   _       => config.get_color("red"),
                };

                let changes = vec![
                    StatusChange::Text(text),
                    StatusChange::Color(color),
                    StatusChange::Size(SizeRequest::Expand),
                ];

                let _ = sx.send(changes);

                let sleep_time = ::std::time::Duration::from_secs(5);
                thread::sleep(sleep_time);
            }
        }

        fun
    }
}

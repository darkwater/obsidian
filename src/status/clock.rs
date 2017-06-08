extern crate time;

use components::*;
use config::Config;
use status::StatusItem;
use std::sync::mpsc;
use std::thread;

pub struct ClockStatusItem;
impl StatusItem for ClockStatusItem {
    fn check_available(&self) -> Result<(), &str> {
        Ok(())
    }

    fn get_update_fun(&self) -> fn(mpsc::Sender<Vec<StatusChange>>, config: &'static Config) {
        fn fun(sx: mpsc::Sender<Vec<StatusChange>>, config: &'static Config) {
            let changes = vec![
                StatusChange::Icon("schedule".to_string()),
            ];

            let _ = sx.send(changes);

            loop {
                let now = time::now();
                let weekday = [ "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat" ][now.tm_wday as usize];

                let color = match now.tm_hour {
                     0... 5 => config.get_color("cyan"),
                     6...11 => config.get_color("blue"),
                    12...17 => config.get_color("green"),
                    18...23 => config.get_color("yellow"),
                    _       => unreachable!()
                };

                let text = format!("{} {} {:02}:{:02}", weekday, now.tm_mday, now.tm_hour, now.tm_min);

                let changes = vec![
                    StatusChange::Text(text),
                    StatusChange::Color(color),
                    StatusChange::Size(SizeRequest::Expand),
                ];

                let _ = sx.send(changes);

                let sleep_time = ::std::time::Duration::new(59 - now.tm_sec as u64, 1000000000 - now.tm_nsec as u32);
                thread::sleep(sleep_time);
            }
        }

        fun
    }
}

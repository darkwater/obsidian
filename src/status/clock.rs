extern crate time;

use components::*;
use status::StatusItem;
use std::sync::mpsc;
use std::thread;

pub struct ClockStatusItem;
impl StatusItem for ClockStatusItem {
    fn check_available(&self) -> bool {
        true
    }

    fn get_update_fun(&self) -> fn(mpsc::Sender<Vec<StatusChange>>) {
        fn fun(sx: mpsc::Sender<Vec<StatusChange>>) {
            let changes = vec![
                StatusChange::Text("Mmm 00 00:00".to_string()),
                StatusChange::Size(SizeRequest::Set)
            ];

            let _ = sx.send(changes);

            loop {
                let now = time::now();
                let weekday = [ "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat" ][now.tm_wday as usize];

                let color = match now.tm_hour {
                     0... 5 => (0.43, 0.53, 0.55, 1.0),
                     6...11 => (0.49, 0.76, 0.81, 1.0),
                    12...17 => (0.72, 0.84, 0.55, 1.0),
                    18...23 => (0.88, 0.67, 0.36, 1.0),
                    _       => (1.0,  1.0,  1.0,  1.0)
                };

                let text = format!("{} {} {:02}:{:02}", weekday, now.tm_mday, now.tm_hour, now.tm_min);

                let changes = vec![
                    StatusChange::Text(text),
                    StatusChange::Color(color)
                ];

                let _ = sx.send(changes);

                let sleep_time = ::std::time::Duration::new(59 - now.tm_sec as u64, 1000000000 - now.tm_nsec as u32);
                thread::sleep(sleep_time);
            }
        }

        fun
    }
}

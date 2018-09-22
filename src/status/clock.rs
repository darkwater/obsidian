extern crate time;

use std::thread;

use relm_core::Sender;
use config::Config;

use ::monitor::*;

pub struct Clock;

impl Default for Clock {
    fn default() -> Self {
        Clock
    }
}

impl Monitor for Clock {
    fn start(self, config: &'static Config, channel: Sender<MonitorMsg>) {
        thread::spawn(move || {
            loop {
                let now = ::time::now();
                let weekday = [ "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat" ][now.tm_wday as usize];

                let color = match now.tm_hour {
                     0... 5 => config.get_color("cyan"),
                     6...11 => config.get_color("blue"),
                    12...17 => config.get_color("green"),
                    18...23 => config.get_color("yellow"),
                    _       => unreachable!()
                };

                let important = now.tm_nsec % 2 == 0;
                let text = format!("{} {} {:02}:{:02}", weekday, now.tm_mday, now.tm_hour, now.tm_min);
                println!("{} / important: {}", text, important);
                channel.send(MonitorMsg::SetText(text));
                channel.send(MonitorMsg::SetColor(color));
                channel.send(MonitorMsg::SetRelevance(Relevance::Urgent));

                let sleep_time = ::std::time::Duration::new(59 - now.tm_sec as u64 % 60, 1000000000 - now.tm_nsec as u32);
                thread::sleep(sleep_time);
            }
        });
    }
}

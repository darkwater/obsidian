use std::thread;

use relm::{Channel, Component, ContainerWidget, Relm, Update, UpdateNew, Widget};
use relm_core::Sender;

use ::manager::*;

#[derive(Debug)]
pub enum MonitorMsg {
    SetText(String),
    SetRelevance(Relevance),
}

#[derive(Clone, Debug)]
pub enum Relevance {
    Urgent,
    Background,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisplayLocation {
    Bar,
    Popup,
    Hidden,
}

pub trait Monitor {
    fn start(&self, channel: Sender<MonitorMsg>);
}

pub struct Clock;

impl Clock {
    pub fn new() -> Self {
        Clock
    }
}

impl Monitor for Clock {
    fn start(&self, channel: Sender<MonitorMsg>) {
        thread::spawn(move || {
            loop {
                let now = ::time::now();
                let weekday = [ "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat" ][now.tm_wday as usize];

                // let color = match now.tm_hour {
                //      0... 5 => config.get_color("cyan"),
                //      6...11 => config.get_color("blue"),
                //     12...17 => config.get_color("green"),
                //     18...23 => config.get_color("yellow"),
                //     _       => unreachable!()
                // };

                let important = now.tm_nsec % 2 == 0;
                let text = format!("{} {} {:02}:{:02}", weekday, now.tm_mday, now.tm_hour, now.tm_min);
                println!("{} / important: {}", text, important);
                channel.send(MonitorMsg::SetText(text));

                channel.send(MonitorMsg::SetRelevance(match important {
                    true  => Relevance::Urgent,
                    false => Relevance::Background,
                }));

                // let changes = vec![
                //     StatusChange::Text(text),
                //     StatusChange::Color(color),
                //     StatusChange::Size(SizeRequest::Expand),
                // ];

                // let _ = sx.send(changes);

                let sleep_time = ::std::time::Duration::new(4 - now.tm_sec as u64 % 5, 1000000000 - now.tm_nsec as u32);
                thread::sleep(sleep_time);
            }
        });
    }
}

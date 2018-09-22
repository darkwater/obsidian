use relm_core::Sender;

use ::color::Color;
use ::config::Config;

#[derive(Debug)]
pub enum MonitorMsg {
    SetText(String),
    SetColor(Color),
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
    fn start(self, config: &'static Config, channel: Sender<MonitorMsg>);
}

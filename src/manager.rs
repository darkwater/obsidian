use relm::{Channel, Relm, Update, UpdateNew};
use relm_core::Sender;

use ::color::Color;
use ::config::Config;
use ::monitor::*;
use ::status::*;

pub struct ManagerModel {
    monitors: Vec<MonitorState>,
    channels: Vec<Channel<MonitorMsg>>,
}

#[derive(Debug, Msg)]
pub enum ManagerMsg {
    RecvMsg(usize, MonitorMsg),
    DisplayUpdate(usize, MonitorState),
}

pub struct Manager {
    relm:  Relm<Manager>,
    model: ManagerModel,
}

#[derive(Clone, Debug)]
pub struct MonitorState {
    pub text:      String,
    pub color:     Color,
    pub relevance: Relevance,
    pub location:  DisplayLocation,
}

#[inline(always)]
fn create_channel(relm: &Relm<Manager>, idx: usize) -> (Channel<MonitorMsg>, Sender<MonitorMsg>) {
    let stream = relm.stream().clone();
    Channel::new(move |msg| {
        stream.emit(ManagerMsg::RecvMsg(idx, msg));
    })
}

#[inline(always)]
fn empty_state() -> MonitorState {
    MonitorState {
        text:      String::new(),
        color:     Color::white(),
        relevance: Relevance::Background,
        location:  DisplayLocation::Hidden,
    }
}

impl Update for Manager {
    type Model      = ManagerModel;
    type ModelParam = &'static Config;
    type Msg        = ManagerMsg;

    fn model(relm: &Relm<Self>, config: Self::ModelParam) -> Self::Model {
        let mut monitors = vec![];
        let mut channels = vec![];

        let battery = Battery::default();
        let (ch, sx) = create_channel(relm, monitors.len());
        battery.start(config, sx);
        monitors.push(empty_state());
        channels.push(ch);

        let clock = Clock::default();
        let (ch, sx) = create_channel(relm, monitors.len());
        clock.start(config, sx);
        monitors.push(empty_state());
        channels.push(ch);

        ManagerModel {
            monitors, channels,
        }
    }

    fn update(&mut self, msg: Self::Msg) {
        println!("{:#?}", msg);
        use self::ManagerMsg::*;
        match msg {
            RecvMsg(i, m)   => self.recv_msg(i, m),
            DisplayUpdate(_, _) => (), // handled by parent
        }
    }
}

impl Manager {
    fn recv_msg(&mut self, idx: usize, msg: MonitorMsg) {
        let state = &mut self.model.monitors[idx];

        match msg {
            MonitorMsg::SetText(s) => state.text = s,
            MonitorMsg::SetColor(c) => state.color = c,
            MonitorMsg::SetRelevance(r) => {
                state.location = match r {
                    Relevance::Urgent     => DisplayLocation::Bar,
                    Relevance::Background => DisplayLocation::Popup,
                };

                state.relevance = r;
            }
        }

        self.relm.stream().emit(ManagerMsg::DisplayUpdate(idx, state.clone()));
    }
}

impl UpdateNew for Manager {
    fn new(relm: &Relm<Self>, model: Self::Model) -> Self {
        Manager {
            relm: relm.clone(),
            model,
        }
    }
}

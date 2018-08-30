use std::mem::transmute;

use relm::{Channel, Component, ContainerWidget, Relm, Update, UpdateNew, Widget};
use relm_core::Sender;

use ::monitor::*;

pub struct ManagerModel {
    monitors: Vec<Option<(Box<dyn Monitor>, MonitorState)>>,
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
        relevance: Relevance::Background,
        location:  DisplayLocation::Hidden,
    }
}

impl Update for Manager {
    type Model      = ManagerModel;
    type ModelParam = ();
    type Msg        = ManagerMsg;

    fn model(relm: &Relm<Self>, _params: Self::ModelParam) -> Self::Model {
        let mut monitors: Vec<Option<(Box<dyn Monitor>, MonitorState)>> = vec![];

        let clock = Clock::new();
        let (ch, sx) = create_channel(relm, monitors.len());
        clock.start(sx);
        monitors.push(Some((Box::new(clock), empty_state())));

        let clock = Clock::new();
        let (ch, sx) = create_channel(relm, monitors.len());
        clock.start(sx);
        monitors.push(Some((Box::new(clock), empty_state())));

        ManagerModel {
            monitors,
        }
    }

    fn update(&mut self, msg: Self::Msg) {
        use self::ManagerMsg::*;
        match msg {
            RecvMsg(i, m)   => self.recv_msg(i, m),
            DisplayUpdate(_, _) => (), // handled by parent
        }
    }
}

impl Manager {
    fn recv_msg(&mut self, idx: usize, msg: MonitorMsg) {
        let state = &mut self.model.monitors[idx].as_mut().expect("removed monitor still sends updates").1;

        match msg {
            MonitorMsg::SetText(s) => state.text = s,
            MonitorMsg::SetRelevance(r) => {
                let new_location = match r {
                    Relevance::Urgent     => DisplayLocation::Bar,
                    Relevance::Background => DisplayLocation::Popup,
                };

                state.relevance = r;

                if state.location != new_location {
                    state.location = new_location;

                    self.relm.stream().emit(ManagerMsg::DisplayUpdate(idx, state.clone()));
                }
            }
        }
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

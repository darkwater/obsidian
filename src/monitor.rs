use std::thread;

use relm::{Channel, Component, ContainerWidget, Relm, Update, UpdateNew, Widget};
use relm_core::Sender;

pub trait Monitor {
    fn start(&self, Sender<MonitorMsg>);
}

pub struct MonitorIfaceModel {
    relm:     Relm<MonitorIface>,
    monitors: Vec<Option<(Box<dyn Monitor>, MonitorState)>>,
}

#[derive(Debug)]
struct MonitorState {
    text: String,
}

#[derive(Msg)]
pub enum MonitorIfaceMsg {
    RecvMsg(usize, MonitorMsg),
}

#[derive(Debug)]
pub enum MonitorMsg {
    SetText(String),
}

pub struct MonitorIface {
    model: MonitorIfaceModel,
}

#[inline(always)]
fn create_channel(relm: &Relm<MonitorIface>, idx: usize) -> (Channel<MonitorMsg>, Sender<MonitorMsg>) {
    let stream = relm.stream().clone();
    Channel::new(move |msg| {
        stream.emit(MonitorIfaceMsg::RecvMsg(idx, msg));
    })
}

#[inline(always)]
fn empty_state() -> MonitorState {
    MonitorState {
        text: String::new(),
    }
}

impl Update for MonitorIface {
    type Model      = MonitorIfaceModel;
    type ModelParam = ();
    type Msg        = MonitorIfaceMsg;

    fn model(relm: &Relm<Self>, _params: Self::ModelParam) -> Self::Model {
        let mut monitors: Vec<Option<(Box<dyn Monitor>, MonitorState)>> = vec![];

        let clock = Clock::new();
        let (ch, sx) = create_channel(relm, monitors.len());
        clock.start(sx);
        monitors.push(Some((Box::new(clock), empty_state())));

        MonitorIfaceModel {
            relm: relm.clone(),
            monitors,
        }
    }

    fn update(&mut self, msg: Self::Msg) {
        match msg {
            MonitorIfaceMsg::RecvMsg(i, m) => self.recv_msg(i, m),
        }
    }
}

impl MonitorIface {
    fn recv_msg(&mut self, idx: usize, msg: MonitorMsg) {
        let state = &mut self.model.monitors[idx].as_mut().expect("removed monitor still sends updates").1;

        match msg {
            MonitorMsg::SetText(s) => state.text = s,
        }

        println!("{:#?}", state);
    }
}

impl UpdateNew for MonitorIface {
    fn new(_relm: &Relm<Self>, model: Self::Model) -> Self {
        MonitorIface {
            model
        }
    }
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
                //     0... 5 => config.get_color("cyan"),
                //     6...11 => config.get_color("blue"),
                //     12...17 => config.get_color("green"),
                //     18...23 => config.get_color("yellow"),
                //     _       => unreachable!()
                // };

                let text = format!("{} {} {:02}:{:02}", weekday, now.tm_mday, now.tm_hour, now.tm_min);
                println!("thread: {}", text);
                channel.send(MonitorMsg::SetText(text));

                // let changes = vec![
                //     StatusChange::Text(text),
                //     StatusChange::Color(color),
                //     StatusChange::Size(SizeRequest::Expand),
                // ];

                // let _ = sx.send(changes);

                let sleep_time = ::std::time::Duration::new(59 - now.tm_sec as u64, 1000000000 - now.tm_nsec as u32);
                thread::sleep(sleep_time);
            }
        });
    }
}

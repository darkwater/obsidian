extern crate cairo;
extern crate gdk;
extern crate gdk_sys;
extern crate gtk;
extern crate i3ipc;
extern crate time;

use std::cell::RefCell;
use std::{cmp, thread};
use std::ops::Range;
use std::rc::Rc;

use gtk::prelude::*;
use i3ipc::{I3Connection, I3EventListener, Subscription};
use relm::{Channel, Relm, Update, Widget};

pub struct WorkspaceModel {
    channel: Channel<WorkspaceMsg>,
    foo: f64
}

pub struct WorkspaceComponent {
    model:  Rc<RefCell<WorkspaceModel>>,
    widget: gtk::DrawingArea,
}

#[derive(Debug, Msg)]
pub enum WorkspaceMsg {
    Foo,
}

impl WorkspaceComponent {
    fn render(model: &WorkspaceModel, widget: &gtk::DrawingArea, cx: &cairo::Context) {
        let width  = widget.get_allocated_width()  as f64;
        let height = widget.get_allocated_height() as f64;

        let (r, g, b, a) = (0.11, 0.12, 0.23, 0.92);
        cx.set_source_rgba(r, g, b, a);
        cx.rectangle(model.foo, 0.0, height, height);
        cx.fill();
    }
}

impl Update for WorkspaceComponent {
    type Model = WorkspaceModel;
    type ModelParam = ();
    type Msg = WorkspaceMsg;

    // Return the initial model.
    fn model(relm: &Relm<Self>, _param: Self::ModelParam) -> Self::Model {
        let stream = relm.stream().clone();

        let (channel, sx) = Channel::new(move |msg| {
            stream.emit(msg);
        });

        thread::spawn(move || {
            let mut i3       = I3Connection::connect().unwrap();
            let mut listener = I3EventListener::connect().unwrap();

            let subs = [ Subscription::Workspace ];
            listener.subscribe(&subs).unwrap();
            let mut listener = listener.listen();

            loop {
                /// Turns workspace names such as 1-2 (screen-workspace) into a tuple of the numbers
                fn parse_workspace_name(name: &str) -> Result<(i64, i64), &'static str> {
                    if name.len() < 3                     { return Err("name too short"); }
                    if name.len() != name.chars().count() { return Err("name contains multibyte characters"); }

                    let (screen, workspace) = name.split_at(1);
                    let screen    = screen.parse().map_err(|_| "invalid workspace name")?;
                    let workspace = (&workspace[1..]).parse().map_err(|_| "invalid workspace name")?;

                    Ok((screen, workspace))
                }

                let mut res = i3.get_workspaces().unwrap().workspaces.iter().map(|workspace| {
                    let res = parse_workspace_name(&workspace.name);
                    res
                }).collect::<Result<Vec<_>, _>>();

                if let Ok(ref mut workspaces) = res {
                    (*workspaces).sort();
                }

                println!("{:#?}", res);

                sx.send(WorkspaceMsg::Foo);

                listener.next();
            }
        });

        WorkspaceModel {
            channel,
            foo: 0.0,
        }
    }

    fn update(&mut self, msg: Self::Msg) {
        use self::WorkspaceMsg::*;
        match msg {
            Foo => {
                let mut model = self.model.borrow_mut();
                model.foo += 10.0;
            },
        }
        self.widget.queue_draw();
    }

    fn subscriptions(&mut self, relm: &Relm<Self>) {
    }
}

impl Widget for WorkspaceComponent {
    type Root = gtk::DrawingArea;

    fn root(&self) -> Self::Root {
        self.widget.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let widget = gtk::DrawingArea::new();
        let model = Rc::new(RefCell::new(model));

        widget.connect_draw(clone!(model => move |widget, cx| {
            WorkspaceComponent::render(&model.borrow(), widget, cx);
            Inhibit(false)
        }));

        WorkspaceComponent {
            model,
            widget,
        }
    }
}

// impl WorkspaceComponent {
//     pub fn new() -> Rc<RefCell<Self>> {
//         let widget = gtk::DrawingArea::new();
//         widget.set_size_request(100, -1);
//         widget.set_vexpand(true);

//         // Tell gtk we actually want to receive the events
//         let mut events = widget.get_events();
//         events |= gdk_sys::GDK_BUTTON_PRESS_MASK.bits() as i32;
//         events |= gdk_sys::GDK_BUTTON_RELEASE_MASK.bits() as i32;
//         widget.set_events(events);

//         let workspaces_component = Rc::new(RefCell::new(WorkspaceComponent {
//             widget: widget,
//             workspaces: vec![]
//         }));

//         let minimum_workspaces_for_screen = vec![ 4, 2, 1 ];
//         let screen_order = vec![ 1, 0, 2 ]; // position in list is screen_order[monitor_index]
//                                             // [ 1, 2, 0 ] => screen[0] is displayed in position 1
//                                             //                screen[1] is displayed in position 2
//                                             //                screen[2] is displayed in position 0

//         {
//             let ref widget = workspaces_component.borrow().widget;

//             widget.connect_button_release_event(clone!(workspaces_component => move |widget, event| {
//                 workspaces_component.borrow().button_release(widget, event)
//             }));

//             widget.connect_draw(clone!(workspaces_component => move |widget, cx| {
//                 workspaces_component.borrow().draw(widget, cx)
//             }));
//         }

//         enum Message {
//             Workspace(Vec<Workspace>),
//             Quit
//         };

//         let (sx, rx) = channel::<Message>();

//         thread::spawn(move || {
//             let mut i3 = i3ipc::I3Connection::connect().unwrap();
//             let mut listener = I3EventListener::connect().unwrap();

//             let subs = [ Subscription::Workspace ];
//             listener.subscribe(&subs).unwrap();
//             let mut listener = listener.listen();

//             loop {
//                 /// Turns workspace names such as 1-2 (screen-workspace) into a tuple of the numbers
//                 fn parse_workspace_name(name: &str) -> Result<(i64, i64), &'static str> {
//                     if name.len() < 3                     { return Err("name too short"); }
//                     if name.len() != name.chars().count() { return Err("name contains multibyte characters"); }

//                     let (screen, workspace) = name.split_at(1);
//                     let screen    = screen.parse().map_err(|_| "invalid workspace name")?;
//                     let workspace = (&workspace[1..]).parse().map_err(|_| "invalid workspace name")?;

//                     Ok((screen, workspace))
//                 }

//                 let mut workspaces = i3.get_workspaces().unwrap().workspaces.iter().map(|workspace| {
//                     let (screen, index) = parse_workspace_name(&workspace.name).expect("invalid workspace name");
//                     let position = screen_order[screen as usize - 1];
//                 }).collect::<Vec<_>>();
//             }
//         });

//         workspaces_component
//     }

//     fn button_release(&self, widget: &gtk::DrawingArea, event: &gdk::EventButton) -> gtk::Inhibit {
//         if event.get_button() != 1 { return Inhibit(false) }

//         let (x, y) = event.get_position();

//         if 0.0 > y || y > widget.get_allocated_height() as f64 {
//             return Inhibit(true)
//         }

//         for workspace in self.workspaces.iter() {
//             if workspace.position.as_ref().map_or(false, |pos| pos.contains(x)) {
//                 let mut i3 = i3ipc::I3Connection::connect().unwrap();
//                 let _ = i3.command(&format!("workspace {}-{}", workspace.screen, workspace.index));

//                 break;
//             }
//         }

//         Inhibit(true)
//     }

//     fn draw(&self, widget: &gtk::DrawingArea, context: &cairo::Context) -> gtk::Inhibit {
//         if self.workspaces.len() == 0 { return Inhibit(false) }

//         let height = widget.get_allocated_height() as f64;
//         let workspace_height = height * 0.25;
//         let skew_ratio = 0.2;
//         let skew = workspace_height * skew_ratio;

//         let first_workspace = self.workspaces.first().unwrap() as *const Workspace;
//         let last_workspace  = self.workspaces.last().unwrap() as *const Workspace;

//         context.set_line_width(1.0);

//         let top    = (height / 2.0 - workspace_height / 2.0).ceil();
//         let bottom = (height / 2.0 + workspace_height / 2.0).floor();

//         for workspace in self.workspaces.as_slice() {
//             if workspace.position.is_none() { unreachable!("positions should've been calculated at this point") }
//             let position = workspace.position.as_ref().unwrap();

//             let mut left_top     = position.start;
//             let mut left_bottom  = left_top;
//             let mut right_top    = position.end;
//             let mut right_bottom = right_top;

//             if workspace as *const Workspace != first_workspace { left_top  += skew; left_bottom  -= skew; }
//             if workspace as *const Workspace != last_workspace  { right_top += skew; right_bottom -= skew; }

//             context.move_to(left_top - 0.5,     top - 0.5);
//             context.line_to(right_top + 0.5,    top - 0.5);
//             context.line_to(right_bottom + 0.5, bottom + 0.5);
//             context.line_to(left_bottom - 0.5,  bottom + 0.5);
//             context.close_path();

//             // Stroke colors
//             if workspace.urgent  { context.set_source_rgba(1.0, 0.7, 0.0, 0.9) } else
//             if workspace.focused { context.set_source_rgba(1.0, 1.0, 1.0, 1.0) } else
//             if workspace.visible { context.set_source_rgba(1.0, 1.0, 1.0, 0.7) } else
//                                  { context.set_source_rgba(1.0, 1.0, 1.0, 0.3) }
//             context.stroke();

//             context.move_to(left_top,     top);
//             context.line_to(right_top,    top);
//             context.line_to(right_bottom, bottom);
//             context.line_to(left_bottom,  bottom);

//             // Fill colors
//             if workspace.urgent   { context.set_source_rgba(1.0, 0.6, 0.0, 0.7) } else
//             if workspace.focused  { context.set_source_rgba(1.0, 1.0, 1.0, 0.8) } else
//             if !workspace.phantom { context.set_source_rgba(1.0, 1.0, 1.0, 0.3) } else
//                                   { context.set_source_rgba(0.0, 0.0, 0.0, 0.2) }
//             context.fill();
//         }

//         Inhibit(false)
//     }
// }

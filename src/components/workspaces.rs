extern crate cairo;
extern crate gdk;
extern crate gdk_sys;
extern crate gtk;
extern crate i3ipc;
extern crate time;

use gtk::prelude::*;
use i3ipc::event::Event;
use i3ipc::I3EventListener;
use i3ipc::Subscription;
use std::cell::RefCell;
use std::io::{BufReader, BufRead};
use std::cmp;
use std::ops::Range;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::thread;

// Good luck working with this code, future me! (Or poor lost soul who stumbled upon this file.)
// I gave up keeping the code nice and clean about halfway converting obsidian from bspwm to i3
// support. It's best to just nuke this file and start over.

pub struct WorkspacesComponent {
    pub widget: gtk::DrawingArea,
    workspaces: Vec<Workspace>
}

#[derive(Debug)]
struct Workspace {

    /// The screen this workspace belongs to.
    screen: i64,

    /// The displayed position of this screen from left to right.
    screen_position: i64,

    /// The index of this workspace within its screen.
    index: i64,

    /// Phantom workspaces are not reported by i3; we use them as filling to have a continuous list
    /// of workspaces (eg. i3 reports workspaces 1 and 3, 2 and 4 are phantom if the user wants a
    /// minimum of 4 to be shown)
    phantom: bool,

    visible: bool,
    focused: bool,
    urgent: bool,

    /// The X position of the left and right sides of this workspace.
    position: Option<Range<f64>>,
}

impl cmp::PartialEq for Workspace {
    fn eq(&self, other: &Workspace) -> bool {
        (self.screen_position, self.index).eq(&(other.screen_position, other.index))
    }
}

impl cmp::Eq for Workspace {}

impl cmp::PartialOrd for Workspace {
    fn partial_cmp(&self, other: &Workspace) -> Option<cmp::Ordering> {
        (self.screen_position, self.index).partial_cmp(&(other.screen_position, other.index))
    }
}

impl cmp::Ord for Workspace {
    fn cmp(&self, other: &Workspace) -> cmp::Ordering {
        (self.screen_position, self.index).cmp(&(other.screen_position, other.index))
    }
}

impl WorkspacesComponent {
    pub fn new() -> Rc<RefCell<Self>> {
        let widget = gtk::DrawingArea::new();
        widget.set_size_request(100, -1);
        widget.set_vexpand(true);

        // Tell gtk we actually want to receive the events
        let mut events = widget.get_events();
        events |= gdk_sys::GDK_BUTTON_PRESS_MASK.bits() as i32;
        events |= gdk_sys::GDK_BUTTON_RELEASE_MASK.bits() as i32;
        widget.set_events(events);

        let workspaces_component = Rc::new(RefCell::new(WorkspacesComponent {
            widget: widget,
            workspaces: vec![]
        }));

        let minimum_workspaces_for_screen = vec![ 4, 2, 2 ];
        let screen_order = vec![ 1, 0, 2 ]; // position in list is screen_order[monitor_index]
                                            // [ 1, 2, 0 ] => screen[0] is displayed in position 1
                                            //                screen[1] is displayed in position 2
                                            //                screen[2] is displayed in position 0

        {
            let ref widget = workspaces_component.borrow().widget;

            widget.connect_button_release_event(clone!(workspaces_component => move |widget, event| {
                workspaces_component.borrow().button_release(widget, event)
            }));

            widget.connect_draw(clone!(workspaces_component => move |widget, cx| {
                workspaces_component.borrow().draw(widget, cx)
            }));
        }

        enum Message {
            Workspaces(Vec<Workspace>),
            Quit
        };

        let (sx, rx) = channel::<Message>();

        gtk::timeout_add(10, clone!(workspaces_component => move || {
            if let Ok(msg) = rx.try_recv() {
                let workspaces;

                match msg {
                    Message::Workspaces(w) => workspaces = w,
                    Message::Quit => {
                        gtk::main_quit();
                        return Continue(false);
                    }
                }

                let workspace_width = 35.0;
                let workspace_padding = 6.0;
                let screen_padding = 15.0;

                let mut last_screen = 0;

                let mut workspace_left = 0.0;

                let workspaces = workspaces.iter().map(|workspace| {
                    if last_screen != workspace.screen {
                        workspace_left += screen_padding;
                        last_screen = workspace.screen;
                    }

                    let left = workspace_left;
                    let right = left + workspace_width;

                    workspace_left += workspace_width + workspace_padding;

                    Workspace {
                        position: Some(left..right),
                        .. *workspace
                    }
                }).collect();

                let mut workspaces_component = workspaces_component.borrow_mut();
                workspaces_component.workspaces = workspaces;
                workspaces_component.widget.queue_draw();
                workspaces_component.widget.set_size_request(workspace_left as i32, -1);
            }

            Continue(true)
        }));

        thread::spawn(move || {
            let mut i3 = i3ipc::I3Connection::connect().unwrap();
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

                let mut workspaces = i3.get_workspaces().unwrap().workspaces.iter().map(|workspace| {
                    let (screen, index) = parse_workspace_name(&workspace.name).expect("invalid workspace name");
                    let position = screen_order[screen as usize - 1];

                    Workspace {
                        screen: screen,
                        screen_position: position,
                        index: index,
                        phantom: false,
                        visible: workspace.visible,
                        focused: workspace.focused,
                        urgent: workspace.urgent,
                        position: None
                    }
                }).collect::<Vec<_>>();

                workspaces.sort();

                let capacity: i64 = minimum_workspaces_for_screen.iter().sum();
                let mut result_workspaces = Vec::with_capacity(capacity as usize);
                let mut last_screen = workspaces[0].screen;
                let mut last_workspace = 0;
                let mut iter = workspaces.into_iter();
                loop {
                    let next = iter.next();

                    // Fill until minimum amount
                    if next.as_ref().map_or(last_screen + 1, |w| w.screen) != last_screen {
                        let position = screen_order[last_screen as usize - 1];

                        for n in (last_workspace + 1)...minimum_workspaces_for_screen[last_screen as usize - 1] {
                            result_workspaces.push(Workspace {
                                screen: last_screen,
                                screen_position: position,
                                index: n,
                                phantom: true,
                                visible: false,
                                focused: false,
                                urgent: false,
                                position: None
                            });
                        }

                        last_workspace = 0;
                    }

                    // Break if this is the end of the list
                    if next.is_none() {
                        break
                    }

                    let workspace = next.unwrap();

                    // Fill in missing workspaces between this and last
                    let position = screen_order[workspace.screen as usize - 1];

                    for n in (last_workspace + 1)..workspace.index {
                        result_workspaces.push(Workspace {
                            screen: workspace.screen,
                            screen_position: position,
                            index: n,
                            phantom: true,
                            visible: false,
                            focused: false,
                            urgent: false,
                            position: None
                        });
                    }

                    last_screen = workspace.screen;
                    last_workspace = workspace.index;

                    result_workspaces.push(workspace);
                }

                result_workspaces.sort();

                let _ = sx.send(Message::Workspaces(result_workspaces));

                if let Some(Err(_)) = listener.next() {
                    sx.send(Message::Quit);
                }
            }
        });

        workspaces_component
    }

    fn button_release(&self, widget: &gtk::DrawingArea, event: &gdk::EventButton) -> gtk::Inhibit {
        if event.get_button() != 1 { return Inhibit(false) }

        let (x, y) = event.get_position();

        if 0.0 > y || y > widget.get_allocated_height() as f64 {
            return Inhibit(true)
        }

        for workspace in self.workspaces.iter() {
            if workspace.position.as_ref().map_or(false, |pos| pos.contains(x)) {
                let mut i3 = i3ipc::I3Connection::connect().unwrap();
                let _ = i3.command(&format!("workspace {}-{}", workspace.screen, workspace.index));

                break;
            }
        }

        Inhibit(true)
    }

    fn draw(&self, widget: &gtk::DrawingArea, context: &cairo::Context) -> gtk::Inhibit {
        if self.workspaces.len() == 0 { return Inhibit(false) }

        let height = widget.get_allocated_height() as f64;
        let workspace_height = height * 0.25;
        let skew_ratio = 0.2;
        let skew = workspace_height * skew_ratio;

        let first_workspace = self.workspaces.first().unwrap() as *const Workspace;
        let last_workspace  = self.workspaces.last().unwrap() as *const Workspace;

        context.set_line_width(1.0);

        let top    = (height / 2.0 - workspace_height / 2.0).ceil();
        let bottom = (height / 2.0 + workspace_height / 2.0).floor();

        for workspace in self.workspaces.as_slice() {
            if workspace.position.is_none() { unreachable!("positions should've been calculated at this point") }
            let position = workspace.position.as_ref().unwrap();

            let mut left_top     = position.start;
            let mut left_bottom  = left_top;
            let mut right_top    = position.end;
            let mut right_bottom = right_top;

            if workspace as *const Workspace != first_workspace { left_top  += skew; left_bottom  -= skew; }
            if workspace as *const Workspace != last_workspace  { right_top += skew; right_bottom -= skew; }

            context.move_to(left_top - 0.5,     top - 0.5);
            context.line_to(right_top + 0.5,    top - 0.5);
            context.line_to(right_bottom + 0.5, bottom + 0.5);
            context.line_to(left_bottom - 0.5,  bottom + 0.5);
            context.close_path();

            // Stroke colors
            if workspace.urgent  { context.set_source_rgba(1.0, 0.7, 0.0, 0.9) } else
            if workspace.focused { context.set_source_rgba(1.0, 1.0, 1.0, 1.0) } else
            if workspace.visible { context.set_source_rgba(1.0, 1.0, 1.0, 0.7) } else
                                 { context.set_source_rgba(1.0, 1.0, 1.0, 0.3) }
            context.stroke();

            context.move_to(left_top,     top);
            context.line_to(right_top,    top);
            context.line_to(right_bottom, bottom);
            context.line_to(left_bottom,  bottom);

            // Fill colors
            if workspace.urgent   { context.set_source_rgba(1.0, 0.6, 0.0, 0.7) } else
            if workspace.focused  { context.set_source_rgba(1.0, 1.0, 1.0, 0.8) } else
            if !workspace.phantom { context.set_source_rgba(1.0, 1.0, 1.0, 0.3) } else
                                  { context.set_source_rgba(0.0, 0.0, 0.0, 0.2) }
            context.fill();
        }

        Inhibit(false)
    }
}

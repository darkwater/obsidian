extern crate cairo;
extern crate gtk;
extern crate gdk;
extern crate gdk_sys;
extern crate time;

use gtk::prelude::*;
use std::cell::RefCell;
use std::io::{BufReader, BufRead};
use std::ops::Range;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::thread;

pub struct WorkspacesComponent {
    pub widget: gtk::DrawingArea,
    workspaces: Vec<Workspace>
}

struct Workspace {

    /// The index of this workspace across all screens.
    index: i64,

    /// The index of the screen this workspace belongs to.
    screen: i64,

    /// The X position of the left and right sides of this workspace.
    position: Range<f64>,

    /// Whether there are any (urgent) windows on this workspace.
    state: WorkspaceState,

    /// Whether the screen containing this workspace is currently active.
    screen_active: bool,

    /// Whether this workspace is currently active.
    active: bool
}

#[derive(Copy, Clone)]
enum WorkspaceState {
    Free,
    Occupied,
    Urgent
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

        {
            let ref widget = workspaces_component.borrow().widget;

            widget.connect_button_release_event(clone!(workspaces_component => move |widget, event| {
                workspaces_component.borrow().button_release(widget, event)
            }));

            widget.connect_draw(clone!(workspaces_component => move |widget, cx| {
                workspaces_component.borrow().draw(widget, cx)
            }));
        }

        let (sx, rx) = channel::<String>();

        gtk::timeout_add(10, clone!(workspaces_component => move || {
            if let Ok(line) = rx.try_recv() {
                let workspace_width = 35.0;
                let workspace_padding = 6.0;
                let screen_padding = 15.0;

                let mut nscreen = 1;
                let mut nworkspace = 1;

                let mut workspaces: Vec<Workspace> = vec![];
                let mut screen_active = false;

                let mut workspace_left = screen_padding;

                for item in line[1..].split(':') {
                    let mut chars = item.chars();
                    let itype = chars.next().expect("bspc reported an empty item");
                    let ivalue = chars.as_str();

                    match itype {
                        'M' => screen_active = true,
                        'm' => screen_active = false,
                        'F' | 'f' | 'O' | 'o' | 'U' | 'u' => {
                            let left = workspace_left;
                            let right = left + workspace_width;

                            workspaces.push(Workspace {
                                index: nworkspace,
                                screen: nscreen,
                                position: left..right,
                                state: match itype {
                                    'F' | 'f' => WorkspaceState::Free,
                                    'O' | 'o' => WorkspaceState::Occupied,
                                    _         => WorkspaceState::Urgent
                                },
                                screen_active: screen_active,
                                active: match itype {
                                    'F' | 'O' | 'U' => true,
                                    _               => false
                                }
                            });

                            nworkspace += 1;
                            workspace_left += workspace_width + workspace_padding;
                        },
                        'L' => {
                            nscreen += 1;
                            workspace_left += screen_padding;
                        },
                        _ => ()
                    }
                }

                workspace_left -= workspace_padding;

                let mut workspaces_component = workspaces_component.borrow_mut();
                workspaces_component.workspaces = workspaces;
                workspaces_component.widget.queue_draw();
                workspaces_component.widget.set_size_request(workspace_left as i32, -1);
            }

            Continue(true)
        }));

        thread::spawn(move || {
            let bspc = Command::new("bspc")
                .args(&[ "subscribe", "report" ])
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let reader = BufReader::new(bspc.stdout.unwrap());

            // XXX: Depends on 'L' to be the last itype sent
            for line in reader.lines() {
                let line = line.unwrap();

                let _ = sx.send(line);
            }
        });

        workspaces_component
    }

    fn button_release(&self, widget: &gtk::DrawingArea, event: &gdk::EventButton) -> gtk::Inhibit {
        let (x, _) = event.get_position();

        for workspace in self.workspaces.as_slice() {
            if workspace.position.contains(x) {
                let _ = Command::new("bspc")
                    .args(&[ "desktop", &format!("^{}", workspace.index), "-f" ])
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("failed to spawn bspc")
                    .wait()
                    .expect("failed to wait for bspc");

                break;
            }
        }

        Inhibit(false)
    }

    fn draw(&self, widget: &gtk::DrawingArea, context: &cairo::Context) -> gtk::Inhibit {
        if self.workspaces.len() == 0 { return Inhibit(false) }

        let padding = 10.0;
        let height = widget.get_allocated_height() as f64;
        let workspace_height = height - padding * 2.0;
        let skew = 1.0;

        let first_workspace = self.workspaces.first().unwrap() as *const Workspace;
        let last_workspace  = self.workspaces.last().unwrap() as *const Workspace;

        context.set_line_width(1.0);

        let top    = padding;
        let bottom = padding + workspace_height;

        for workspace in self.workspaces.as_slice() {
            let mut left_top     = workspace.position.start;
            let mut left_bottom  = left_top;
            let mut right_top    = workspace.position.end;
            let mut right_bottom = right_top;

            if workspace as *const Workspace != first_workspace { left_top  += skew; left_bottom  -= skew; }
            if workspace as *const Workspace != last_workspace  { right_top += skew; right_bottom -= skew; }

            context.move_to(left_top - 0.5,     top - 0.5);
            context.line_to(right_top + 0.5,    top - 0.5);
            context.line_to(right_bottom + 0.5, bottom + 0.5);
            context.line_to(left_bottom - 0.5,  bottom + 0.5);
            context.close_path();

            // Stroke colors
            match (workspace.screen_active, workspace.active, workspace.state) {
                (_,     _,     WorkspaceState::Urgent)   => context.set_source_rgba(1.0, 0.7, 0.0, 0.9),
                (_,     false, WorkspaceState::Free)     => context.set_source_rgba(1.0, 1.0, 1.0, 0.3),
                (_,     false, WorkspaceState::Occupied) => context.set_source_rgba(1.0, 1.0, 1.0, 0.3),
                (true,  true,  _)                        => context.set_source_rgba(1.0, 1.0, 1.0, 1.0),
                (false, true,  _)                        => context.set_source_rgba(1.0, 1.0, 1.0, 0.7),
            }
            context.stroke();

            context.move_to(left_top,     top);
            context.line_to(right_top,    top);
            context.line_to(right_bottom, bottom);
            context.line_to(left_bottom,  bottom);

            // Fill colors
            match (workspace.screen_active && workspace.active, workspace.state) {
                (_,     WorkspaceState::Urgent)   => context.set_source_rgba(1.0, 0.6, 0.0, 0.7),
                (_,     WorkspaceState::Free)     => context.set_source_rgba(0.0, 0.0, 0.0, 0.2),
                (false, WorkspaceState::Occupied) => context.set_source_rgba(1.0, 1.0, 1.0, 0.3),
                (true,  _)                        => context.set_source_rgba(1.0, 1.0, 1.0, 0.8),
            }
            context.fill();
        }

        Inhibit(false)
    }
}

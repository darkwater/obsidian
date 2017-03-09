extern crate cairo;
extern crate gtk;
extern crate time;

use gtk::prelude::*;
use std::cell::RefCell;
use std::io::{BufReader, BufRead};
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::Mutex;
use std::thread;

pub struct Workspaces {
    pub widget: gtk::DrawingArea,
    screens: Mutex<Vec<Screen>>
}

#[derive(Debug)]
struct Screen {
    index: i64,
    active: bool,
    workspaces: Vec<Workspace>
}

#[derive(Debug)]
struct Workspace {
    index: i64,
    state: WorkspaceState,
    active: bool
}

#[derive(Debug, Copy, Clone)]
enum WorkspaceState {
    Free,
    Occupied,
    Urgent
}

impl Workspaces {
    pub fn new() -> Rc<RefCell<Self>> {
        let widget = gtk::DrawingArea::new();
        widget.set_size_request(350, -1);
        widget.set_vexpand(true);

        let workspaces = Rc::new(RefCell::new(Workspaces {
            widget: widget,
            screens: Mutex::new(vec![])
        }));

        workspaces.borrow().widget.connect_draw(clone!(workspaces => move |widget, cx| {
            workspaces.borrow().draw(widget, cx)
        }));

        thread::spawn(|| {
            let bspc = Command::new("bspc")
                .args(&[ "subscribe", "report" ])
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let reader = BufReader::new(bspc.stdout.unwrap());

            // XXX: Depends on 'L' to be the last itype sent
            for line in reader.lines() {
                let line = line.unwrap();

                let mut nscreen = 1;
                let mut nworkspace = 1;

                let mut screens: Vec<Screen> = vec![];
                let mut workspaces: Vec<Workspace> = vec![];
                let mut screen_active = false;

                for item in line[1..].split(':') {
                    let mut chars = item.chars();
                    let itype = chars.next().expect("bspc reported an empty item");
                    let ivalue = chars.as_str();

                    match itype {
                        'M' => screen_active = true,
                        'm' => screen_active = false,
                        'F' | 'f' | 'O' | 'o' | 'U' | 'u' => {
                            workspaces.push(Workspace {
                                index: nworkspace,
                                state: match itype {
                                    'F' | 'f' => WorkspaceState::Free,
                                    'O' | 'o' => WorkspaceState::Occupied,
                                    _         => WorkspaceState::Urgent
                                },
                                active: match itype {
                                    'F' | 'O' | 'U' => true,
                                    _               => false
                                }
                            });

                            nworkspace += 1;
                        },
                        'L' => {
                            screens.push(Screen {
                                index: nscreen,
                                active: screen_active,
                                workspaces: workspaces
                            });

                            nscreen += 1;
                            workspaces = vec![];
                        },
                        _ => ()
                    }
                }

                println!("{:?}", screens);
            }

            // widget.queue_draw();
        });

        workspaces
    }

    fn draw(&self, widget: &gtk::DrawingArea, context: &cairo::Context) -> gtk::Inhibit {
        let screens = self.screens.lock().unwrap();
        if screens.len() == 0 { return Inhibit(false) }

        let height = widget.get_allocated_height() as f64;
        let padding = 10.0;
        let workspace_width = 20.0;
        let workspace_height = height - padding * 2.0;
        let workspace_padding = 6.0;
        let screen_padding = 15.0;
        let skew = 1.0;

        let first_workspace = screens.first().unwrap().workspaces.first().unwrap() as *const Workspace;
        let last_workspace  = screens.last().unwrap().workspaces.last().unwrap() as *const Workspace;

        context.set_line_width(1.0);

        let mut screen_left = 10.0;
        let num_screens = screens.len();

        for screen in screens.as_slice() {
            let top         = padding;
            let bottom      = padding + workspace_height;

            let mut workspace_left = screen_left;
            let num_workspaces = screen.workspaces.len();

            for workspace in screen.workspaces.as_slice() {
                let mut left_top     = workspace_left;
                let mut left_bottom  = left_top;
                let mut right_top    = left_top + workspace_width;
                let mut right_bottom = right_top;

                if workspace as *const Workspace != first_workspace { left_top  += skew; left_bottom  -= skew; }
                if workspace as *const Workspace != last_workspace  { right_top += skew; right_bottom -= skew; }

                context.move_to(left_top - 0.5,     top - 0.5);
                context.line_to(right_top + 0.5,    top - 0.5);
                context.line_to(right_bottom + 0.5, bottom + 0.5);
                context.line_to(left_bottom - 0.5,  bottom + 0.5);
                context.close_path();

                match (workspace.active, workspace.state) {
                    (_,     WorkspaceState::Urgent)   => context.set_source_rgba(1.0, 0.69, 0.0, 0.9),
                    (false, WorkspaceState::Free)     => context.set_source_rgba(1.0, 1.0,  1.0, 0.3),
                    (false, WorkspaceState::Occupied) => context.set_source_rgba(1.0, 1.0,  1.0, 0.3),
                    (true,  _)                        => context.set_source_rgba(1.0, 1.0,  1.0, 1.0),
                }
                context.stroke();

                context.move_to(left_top,     top);
                context.line_to(right_top,    top);
                context.line_to(right_bottom, bottom);
                context.line_to(left_bottom,  bottom);

                match (workspace.active, workspace.state) {
                    (_,     WorkspaceState::Urgent)   => context.set_source_rgba(1.0, 0.6, 0.0, 0.7),
                    (false, WorkspaceState::Free)     => context.set_source_rgba(0.0, 0.0, 0.0, 0.2),
                    (false, WorkspaceState::Occupied) => context.set_source_rgba(1.0, 1.0, 1.0, 0.3),
                    (true,  _)                        => context.set_source_rgba(1.0, 1.0, 1.0, 0.8),
                }
                context.fill();

                workspace_left += workspace_width + workspace_padding;
            }

            screen_left += workspace_width * num_workspaces as f64 + workspace_padding * (num_workspaces as f64 - 1.0) + screen_padding;
        }

        Inhibit(false)
    }
}

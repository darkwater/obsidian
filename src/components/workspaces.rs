extern crate time;
extern crate cairo;
extern crate gtk;

use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Workspaces {
    pub widget: gtk::DrawingArea,
    screens: Vec<Screen>
}

struct Screen {
    id: i64,
    workspaces: Vec<Workspace>
}

struct Workspace {
    id: i64,
    state: WorkspaceState
}

enum WorkspaceState {
    Empty,
    Used,
    Current
}

impl Workspaces {
    pub fn new() -> Rc<RefCell<Self>> {
        let widget = gtk::DrawingArea::new();
        widget.set_size_request(350, -1);
        widget.set_vexpand(true);

        let workspaces = Rc::new(RefCell::new(Workspaces {
            widget: widget,
            screens: vec![
                Screen { id: 0x00400003, workspaces: vec![
                    Workspace { id: 0x00400008, state: WorkspaceState::Used  },
                    Workspace { id: 0x0040000A, state: WorkspaceState::Used  },
                    Workspace { id: 0x0040000B, state: WorkspaceState::Empty },
                ] },
                Screen { id: 0x00400001, workspaces: vec![
                    Workspace { id: 0x00400008, state: WorkspaceState::Current },
                    Workspace { id: 0x0040000A, state: WorkspaceState::Used    },
                    Workspace { id: 0x0040000B, state: WorkspaceState::Empty   },
                ] },
                Screen { id: 0x00400005, workspaces: vec![
                    Workspace { id: 0x00400007, state: WorkspaceState::Used },
                    Workspace { id: 0x0040000F, state: WorkspaceState::Empty },
                    Workspace { id: 0x00400010, state: WorkspaceState::Empty },
                ] }
            ]
        }));

        workspaces.borrow().widget.connect_draw(clone!(workspaces => move |widget, cx| {
            workspaces.borrow().draw(widget, cx)
        }));

        // gtk::timeout_add(1000, clone!(workspaces => move || {
        //     workspaces.borrow_mut().update();
        //     Continue(true)
        // }));

        workspaces.borrow_mut().update();

        workspaces
    }

    fn update(&mut self) {
        // let now = time::now();
        // let weekday = [ "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat" ][now.tm_wday as usize];

        // self.text = format!("{} {} {:02}:{:02}", weekday, now.tm_mday, now.tm_hour, now.tm_min);
        // self.widget.queue_draw();
    }

    fn draw(&self, widget: &gtk::DrawingArea, context: &cairo::Context) -> gtk::Inhibit {
        let height = widget.get_allocated_height() as f64;
        let padding = 10.0;
        let workspace_width = 20.0;
        let workspace_height = height - padding * 2.0;
        let workspace_padding = 6.0;
        let screen_padding = 15.0;
        let skew = 2.0;

        let first_workspace = self.screens.first().unwrap().workspaces.first().unwrap() as *const Workspace;
        let last_workspace  = self.screens.last().unwrap().workspaces.last().unwrap() as *const Workspace;

        context.set_line_width(1.0);

        let mut screen_left = 10.0;
        let num_screens = self.screens.len();

        for screen in self.screens.as_slice() {
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

                match workspace.state {
                    WorkspaceState::Empty   => context.set_source_rgba(1.0, 1.0, 1.0, 0.3),
                    WorkspaceState::Used    => context.set_source_rgba(1.0, 1.0, 1.0, 0.2),
                    WorkspaceState::Current => context.set_source_rgba(1.0, 1.0, 1.0, 0.9),
                }
                context.stroke();

                context.move_to(left_top,     top);
                context.line_to(right_top,    top);
                context.line_to(right_bottom, bottom);
                context.line_to(left_bottom,  bottom);

                match workspace.state {
                    WorkspaceState::Empty   => context.set_source_rgba(0.0, 0.0, 0.0, 0.2),
                    WorkspaceState::Used    => context.set_source_rgba(1.0, 1.0, 1.0, 0.2),
                    WorkspaceState::Current => context.set_source_rgba(1.0, 1.0, 1.0, 0.7),
                }
                context.fill();

                workspace_left += workspace_width + workspace_padding;
            }

            screen_left += workspace_width * num_workspaces as f64 + workspace_padding * (num_workspaces as f64 - 1.0) + screen_padding;
        }

        Inhibit(false)
    }
}

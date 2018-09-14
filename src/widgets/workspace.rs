extern crate cairo;
extern crate gdk;
extern crate gdk_sys;
extern crate gtk;
extern crate i3ipc;
extern crate time;

use std::cell::RefCell;
use std::ops::Range;
use std::process::Command;
use std::rc::Rc;
use std::thread;

use gtk::prelude::*;
use i3ipc::{I3Connection, I3EventListener, Subscription};
use relm::{Channel, Relm, Update, Widget};

use ::config::Config;

pub struct WorkspaceModel {
    config: &'static Config,
    channel: Channel<WorkspaceMsg>,
    items: Vec<Item>,
}

pub struct WorkspaceWidget {
    model:  Rc<RefCell<WorkspaceModel>>,
    widget: gtk::DrawingArea,
}

#[derive(Debug, Msg)]
pub enum WorkspaceMsg {
    Items(Vec<Item>),
    Click((f64, f64)),
}

#[derive(Debug)]
pub struct Item {
    workspace: (i64, i64),
    position: Range<f64>,
    state: State,
}

#[derive(Debug)]
enum State {
    /// A workspace that doesn't actually exist (has no windows) but should be shown and can be
    /// switched to.
    Phantom,

    /// A regular workspace that has windows in it, but isn't visible.
    Inhibited,

    /// A workspace that's visible but not currently active.
    Visible,

    /// The workspace that is currently active.
    Active,

    /// A workspace with an urgent window in it, even if the workspace is visible or active.
    Urgent,
}

impl WorkspaceWidget {
    fn render(model: &WorkspaceModel, widget: &gtk::DrawingArea, cx: &cairo::Context) {
        let width  = widget.get_allocated_width()  as f64;
        let height = widget.get_allocated_height() as f64;
        let dpi    = model.config.dpi.get();

        if model.items.is_empty() { return }

        let workspace_height = height * 0.25;
        let skew_ratio = 0.2;
        let skew = workspace_height * skew_ratio * dpi;

        let required_width = model.items.last().unwrap().position.end * dpi + skew + 5.0;
        if required_width > width {
            widget.set_size_request(required_width as i32 + 5, height as i32);
            return;
        }

        let first_workspace = model.items.first().unwrap() as *const Item;
        let last_workspace  = model.items.last().unwrap() as *const Item;

        cx.set_line_width((1.0 * dpi).floor());

        let top    = (height / 2.0 - workspace_height / 2.0).ceil();
        let bottom = (height / 2.0 + workspace_height / 2.0).floor();

        for workspace in &model.items {
            let mut left_top     = workspace.position.start * dpi;
            let mut left_bottom  = left_top;
            let mut right_top    = workspace.position.end * dpi;
            let mut right_bottom = right_top;

            if workspace as *const Item != first_workspace { left_top  += skew; left_bottom  -= skew; }
            if workspace as *const Item != last_workspace  { right_top += skew; right_bottom -= skew; }

            cx.move_to(left_top - 0.5,     top - 0.5);
            cx.line_to(right_top + 0.5,    top - 0.5);
            cx.line_to(right_bottom + 0.5, bottom + 0.5);
            cx.line_to(left_bottom - 0.5,  bottom + 0.5);
            cx.close_path();

            match workspace.state {
                State::Urgent    => cx.set_source_rgba(1.0, 0.7, 0.0, 0.9),
                State::Active    => cx.set_source_rgba(1.0, 1.0, 1.0, 1.0),
                State::Visible   => cx.set_source_rgba(1.0, 1.0, 1.0, 0.7),
                State::Inhibited => cx.set_source_rgba(1.0, 1.0, 1.0, 0.3),
                State::Phantom   => cx.set_source_rgba(1.0, 1.0, 1.0, 0.3),
            }

            cx.stroke();

            cx.move_to(left_top,     top);
            cx.line_to(right_top,    top);
            cx.line_to(right_bottom, bottom);
            cx.line_to(left_bottom,  bottom);
            cx.close_path();

            match workspace.state {
                State::Urgent    => cx.set_source_rgba(1.0, 0.6, 0.0, 0.7),
                State::Active    => cx.set_source_rgba(1.0, 1.0, 1.0, 0.8),
                State::Visible   => cx.set_source_rgba(1.0, 1.0, 1.0, 0.3),
                State::Inhibited => cx.set_source_rgba(1.0, 1.0, 1.0, 0.3),
                State::Phantom   => cx.set_source_rgba(0.0, 0.0, 0.0, 0.2),
            }

            cx.fill();
        }
    }

    fn handle_click(&self, (x, _y): (f64, f64)) {
        for item in &self.model.borrow().items {
            if item.position.contains(&x) {
                Command::new("/bin/bash")
                    .arg("-c")
                    .arg(format!("i3 workspace {}-{}", item.workspace.0, item.workspace.1))
                    .spawn()
                    .expect("failed to execute child");
            }
        }
    }
}

impl Update for WorkspaceWidget {
    type Model = WorkspaceModel;
    type ModelParam = &'static Config;
    type Msg = WorkspaceMsg;

    // Return the initial model.
    fn model(relm: &Relm<Self>, config: Self::ModelParam) -> Self::Model {
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

                let res = i3.get_workspaces().unwrap().workspaces.iter().map(|workspace| {
                    parse_workspace_name(&workspace.name)
                        .map(|position| {
                            let state = if workspace.urgent { State::Urgent }
                            else if workspace.focused { State::Active }
                            else if workspace.visible { State::Visible }
                            else { State::Inhibited };

                            (position, state)
                        })
                }).collect::<Result<Vec<_>, _>>();

                let res = res.map(|mut workspaces| {
                    if workspaces.is_empty() { return vec![] }

                    let min_desktops = &[4, 2, 1];
                    let screen_order = &[1, 0, 2];
                    workspaces.sort_by_key(|(pos, _state)| (screen_order[pos.0 as usize - 1], pos.1));

                    let workspaces = workspaces.into_iter().peekable();

                    let item_width = 35.0;
                    let padding    = 6.0;
                    let spacing    = 15.0;

                    let mut items        = vec![];
                    let mut left         = -padding;
                    let mut last_screen  = 0;
                    let mut last_desktop = 0;
                    for (workspace, state) in workspaces {
                        left += padding;
                        if last_screen != workspace.0 {
                            if last_screen != 0 {
                                for n in (last_desktop + 1) .. (min_desktops[last_screen as usize - 1] + 1) {
                                    let workspace = (last_screen, n);

                                    let position = (left) .. (left + item_width);
                                    left += item_width + padding;

                                    let state = State::Phantom;

                                    items.push(Item {
                                        workspace, position, state,
                                    });
                                }
                            }

                            left += spacing;
                            last_desktop = 0;
                        }

                        for n in (last_desktop + 1) .. workspace.1 {
                            let workspace = (workspace.0, n);

                            let position = (left) .. (left + item_width);
                            left += item_width + padding;

                            let state = State::Phantom;

                            items.push(Item {
                                workspace, position, state,
                            });
                        }

                        let position = (left) .. (left + item_width);
                        left += item_width;

                        items.push(Item {
                            workspace, position, state,
                        });

                        last_screen  = workspace.0;
                        last_desktop = workspace.1;
                    }

                    items
                });

                sx.send(WorkspaceMsg::Items(res.unwrap()));

                listener.next();
            }
        });

        WorkspaceModel {
            config,
            items: vec![],
            channel,
        }
    }

    fn update(&mut self, msg: Self::Msg) {
        use self::WorkspaceMsg::*;
        match msg {
            Items(v) => self.model.borrow_mut().items = v,
            Click(e) => self.handle_click(e),
        }
        self.widget.queue_draw();
    }

    fn subscriptions(&mut self, _relm: &Relm<Self>) {
    }
}

impl Widget for WorkspaceWidget {
    type Root = gtk::DrawingArea;

    fn root(&self) -> Self::Root {
        self.widget.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let widget = gtk::DrawingArea::new();
        let model = Rc::new(RefCell::new(model));

        widget.add_events(gdk::EventMask::BUTTON_PRESS_MASK.bits() as i32);
        widget.add_events(gdk::EventMask::BUTTON_RELEASE_MASK.bits() as i32);

        connect!(relm, widget, connect_button_release_event(_, e), return if e.get_button() == 1 {
            (Some(WorkspaceMsg::Click(e.get_position())), Inhibit(true))
        }
        else {
            (None, Inhibit(false))
        });

        widget.connect_draw(clone!(model => move |widget, cx| {
            WorkspaceWidget::render(&model.borrow(), widget, cx);
            Inhibit(false)
        }));

        WorkspaceWidget {
            model,
            widget,
        }
    }
}

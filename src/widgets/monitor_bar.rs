extern crate cairo;
extern crate gdk;
extern crate gtk;
extern crate pango;
extern crate pangocairo;
extern crate relm_core;

use std::cell::RefCell;
use std::mem;
use std::ops::Range;
use std::rc::Rc;

use gtk::prelude::*;
use self::pango::prelude::LayoutExt;
use relm::{Channel, Relm, Update, Widget};

use ::monitor::*;
use ::manager::*;

pub struct MonitorBarModel {
    states: Vec<Option<MonitorState>>,
    displayed: Vec<usize>,
}

pub struct MonitorBarWidget {
    model:  Rc<RefCell<MonitorBarModel>>,
    widget: gtk::DrawingArea,
}

#[derive(Debug, Msg)]
pub enum MonitorBarMsg {
    Click((f64, f64)),
    RecvUpdate(usize, MonitorState),
}

impl MonitorBarWidget {
    fn render(model: &MonitorBarModel, widget: &gtk::DrawingArea, context: &cairo::Context) {
        let _width  = widget.get_allocated_width()  as f64;
        let height = widget.get_allocated_height() as f64;

        let mut text = String::new();
        for item in &model.displayed {
            text.push_str(&format!(" [ item {} ] ", item));
        }

        let font = pango::FontDescription::from_string("Droid Sans Mono 10");
        let layout = pangocairo::functions::create_layout(context).unwrap();
        layout.set_text(&text);
        layout.set_font_description(Some(&font));

        let extents = layout.get_extents().0;
        let (icon_x, icon_y) = (extents.x as f64 / pango::SCALE as f64,
                                extents.y as f64 / pango::SCALE as f64);
        let (icon_width, icon_height) = (extents.width  as f64 / pango::SCALE as f64,
                                         extents.height as f64 / pango::SCALE as f64);

        let x = -icon_x as f64;
        let y = -icon_y as f64 + height / 2.0 - icon_height / 2.0;

        // let Color(r, g, b, a) = self.color;
        context.set_source_rgba(1.0, 1.0, 1.0, 1.0);

        context.move_to(x, y);
        pangocairo::functions::show_layout(&context, &layout);
    }

    fn recv_update(&mut self, idx: usize, new: MonitorState) {
        let model = &mut self.model.borrow_mut();

        if idx >= model.states.len() {
            model.states.resize(idx + 1, None);
        }

        match &mut model.states[idx] {
            Some(ref mut state) => {
                let mut old = new;
                mem::swap(state, &mut old);

                if old.location != state.location {
                    match (&old.location, &state.location) {
                        (_, DisplayLocation::Bar) => Self::show_state(model, idx),
                        (DisplayLocation::Bar, _) => Self::hide_state(model, idx),
                        _ => (),
                    }
                }
            },
            None => {
                model.states[idx] = Some(new);
                if model.states[idx].as_ref().unwrap().location == DisplayLocation::Bar {
                    Self::show_state(model, idx);
                }
            },
        }

        self.widget.queue_draw();
    }

    fn show_state(model: &mut MonitorBarModel, idx: usize) {
        println!("showing");
        model.displayed.insert(0, idx);
    }

    fn hide_state(model: &mut MonitorBarModel, idx: usize) {
        println!("hiding");
        model.displayed.retain(|i| *i != idx);
    }

    fn handle_click(&self, (x, _y): (f64, f64)) {
        // for item in &self.model.borrow().items {
        //     if item.position.contains(&x) {
        //         Command::new("/bin/bash")
        //             .arg("-c")
        //             .arg(format!("i3 workspace {}-{}", item.workspace.0, item.workspace.1))
        //             .spawn()
        //             .expect("failed to execute child");
        //     }
        // }
    }
}

impl Update for MonitorBarWidget {
    type Model = MonitorBarModel;
    type ModelParam = ();
    type Msg = MonitorBarMsg;

    // Return the initial model.
    fn model(relm: &Relm<Self>, _param: Self::ModelParam) -> Self::Model {
        MonitorBarModel {
            states: vec![],
            displayed: vec![],
        }
    }

    fn update(&mut self, msg: Self::Msg) {
        use self::MonitorBarMsg::*;
        match msg {
            Click(e)         => self.handle_click(e),
            RecvUpdate(i, s) => self.recv_update(i, s),
        }
        self.widget.queue_draw();
    }

    fn subscriptions(&mut self, _relm: &Relm<Self>) {
    }
}

impl Widget for MonitorBarWidget {
    type Root = gtk::DrawingArea;

    fn root(&self) -> Self::Root {
        self.widget.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let widget = gtk::DrawingArea::new();
        let model = Rc::new(RefCell::new(model));

        widget.set_hexpand(true);

        widget.add_events(gdk::EventMask::BUTTON_PRESS_MASK.bits() as i32);
        widget.add_events(gdk::EventMask::BUTTON_RELEASE_MASK.bits() as i32);

        connect!(relm, widget, connect_button_release_event(_, e), return if e.get_button() == 1 {
            (Some(MonitorBarMsg::Click(e.get_position())), Inhibit(true))
        }
        else {
            (None, Inhibit(false))
        });

        widget.connect_draw(clone!(model => move |widget, cx| {
            MonitorBarWidget::render(&model.borrow(), widget, cx);
            Inhibit(false)
        }));

        MonitorBarWidget {
            model,
            widget,
        }
    }
}

extern crate cairo;
extern crate gdk;
extern crate gtk;
extern crate libloading as lib;
extern crate pango;
extern crate pangocairo;
extern crate relm_core;

use std::cell::RefCell;
use std::ops::Range;
use std::process::Command;
use std::rc::Rc;
use std::thread;

use gtk::prelude::*;
use self::pango::prelude::LayoutExt;
use i3ipc::{I3Connection, I3EventListener, Subscription};
use relm::{Channel, Relm, Update, Widget};

pub struct PluginModel {
    channel: Channel<String>,
    text:    String,
}

pub struct PluginWidget {
    model:  Rc<RefCell<PluginModel>>,
    widget: gtk::DrawingArea,
}

#[derive(Debug, Msg)]
pub enum PluginMsg {
    Update(String),
}

impl PluginWidget {
    fn render(model: &PluginModel, widget: &gtk::DrawingArea, context: &cairo::Context) {
        let _width = widget.get_allocated_width()  as f64;
        let height = widget.get_allocated_height() as f64;

        let text = &model.text;

        let font = pango::FontDescription::from_string("Droid Sans Mono 12");
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
}

impl Update for PluginWidget {
    type Model = PluginModel;
    type ModelParam = &'static str;
    type Msg = PluginMsg;

    // Return the initial model.
    fn model(relm: &Relm<Self>, param: Self::ModelParam) -> Self::Model {
        let stream = relm.stream().clone();

        let (channel, sx) = Channel::new(move |msg| {
            stream.emit(PluginMsg::Update(msg));
        });

        thread::spawn(move || {
            let lib = lib::Library::new(param).expect("lib load");

            unsafe {
                let plugin_func = lib.get::<lib::Symbol<unsafe extern fn(relm_core::Sender<String>) -> ()>>(b"foo\0").unwrap();
                plugin_func(sx);
            }
        });

        PluginModel {
            text: String::new(),
            channel,
        }
    }

    fn update(&mut self, msg: Self::Msg) {
        use self::PluginMsg::*;
        match msg {
            Update(s) => self.model.borrow_mut().text = s,
        }
        self.widget.queue_draw();
    }

    fn subscriptions(&mut self, _relm: &Relm<Self>) {
    }
}

impl Widget for PluginWidget {
    type Root = gtk::DrawingArea;

    fn root(&self) -> Self::Root {
        self.widget.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let widget = gtk::DrawingArea::new();
        let model = Rc::new(RefCell::new(model));

        widget.add_events(gdk::EventMask::BUTTON_PRESS_MASK.bits() as i32);
        widget.add_events(gdk::EventMask::BUTTON_RELEASE_MASK.bits() as i32);

        widget.connect_draw(clone!(model => move |widget, cx| {
            PluginWidget::render(&model.borrow(), widget, cx);
            Inhibit(false)
        }));

        PluginWidget {
            model, widget,
        }
    }
}

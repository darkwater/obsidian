extern crate cairo;
extern crate gtk;
extern crate gdk;
extern crate gdk_sys;
extern crate time;

use gtk::prelude::*;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;

pub struct LoadComponent {
    pub widget: gtk::DrawingArea,
    loadavg: f64
}

impl LoadComponent {
    pub fn new() -> Rc<RefCell<Self>> {
        let widget = gtk::DrawingArea::new();
        widget.set_size_request(50, -1);
        widget.set_vexpand(true);

        let load_component = Rc::new(RefCell::new(LoadComponent {
            widget: widget,
            loadavg: 0.00
        }));

        load_component.borrow_mut().update();

        {
            let ref mut widget = load_component.borrow_mut().widget;

            // widget.connect_button_release_event(clone!(load_component => move |widget, event| {
            //     load_component.borrow().button_release(widget, event)
            // }));

            widget.connect_draw(clone!(load_component => move |widget, cx| {
                load_component.borrow().draw(widget, cx)
            }));
        }

        let (sx, rx) = channel::<bool>();

        gtk::timeout_add(5 * 1000, clone!(load_component => move || {
            load_component.borrow_mut().update();

            Continue(true)
        }));

        load_component
    }

    fn update(&mut self) {
        let mut file = File::open("/proc/loadavg").expect("Couldn't open /proc/loadavg");
        let mut string = String::with_capacity(32);
        let _ = file.read_to_string(&mut string);
        let mut split = string.split(' ');

        self.loadavg = f64::from_str(&split.nth(1).unwrap()).expect("Expected a float from /proc/loadavg");

        self.widget.queue_draw();
    }

    fn draw(&self, widget: &gtk::DrawingArea, context: &cairo::Context) -> gtk::Inhibit {
        let width  = widget.get_allocated_width()  as f64;
        let height = widget.get_allocated_height() as f64;
        let text = &format!("{:.2}", self.loadavg);

        context.set_font_size(12.0);
        context.select_font_face("Droid Sans Mono",
                                 cairo::enums::FontSlant::Normal,
                                 cairo::enums::FontWeight::Normal);

        let extents = context.text_extents(text);
        let x = width  / 2.0 - extents.width  / 2.0 - extents.x_bearing;
        let y = height / 2.0 - extents.height / 2.0 - extents.y_bearing;

        match self.loadavg {
            0.0...0.1 => context.set_source_rgba(0.2, 1.0, 0.5, 0.95),
            0.1...0.4 => context.set_source_rgba(0.1, 1.0, 0.1, 0.95),
            0.4...0.8 => context.set_source_rgba(1.0, 0.7, 0.0, 0.95),
            _         => context.set_source_rgba(1.0, 0.3, 0.1, 0.95),
        }

        context.move_to(x, y);
        context.show_text(text);

        Inhibit(false)
    }
}

extern crate cairo;
extern crate gtk;
extern crate time;

use self::cairo::Gradient;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Separator {
}

impl Separator {
    pub fn new() -> gtk::DrawingArea {
        let widget = gtk::DrawingArea::new();
        widget.set_vexpand(true);

        widget.set_size_request(1, -1);

        widget.connect_draw(|widget, context| {
            let width  = widget.get_allocated_width()  as f64;
            let height = widget.get_allocated_height() as f64;

            let pattern = cairo::LinearGradient::new(0.0, 0.0, 0.0, height);
            pattern.add_color_stop_rgba(0.3, 1.0, 1.0, 1.0, 0.3);
            pattern.add_color_stop_rgba(0.4, 1.0, 1.0, 1.0, 0.6);
            pattern.add_color_stop_rgba(0.6, 1.0, 1.0, 1.0, 0.6);
            pattern.add_color_stop_rgba(0.7, 1.0, 1.0, 1.0, 0.3);
            context.set_source(&pattern);
            context.rectangle(0.0, height * 0.3, width, height * 0.4);
            context.fill();

            Inhibit(false)
        });

        widget
    }
}

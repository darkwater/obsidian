extern crate time;
extern crate cairo;
extern crate gtk;

use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ClockComponent {
    pub widget: gtk::DrawingArea,
    text: String
}

impl ClockComponent {
    pub fn new() -> Rc<RefCell<Self>> {
        let widget = gtk::DrawingArea::new();
        widget.set_size_request(110, -1);
        widget.set_vexpand(true);

        let clock = Rc::new(RefCell::new(ClockComponent {
            widget: widget,
            text: "00:00:00".to_string()
        }));

        clock.borrow().widget.connect_draw(clone!(clock => move |widget, cx| {
            clock.borrow().draw(widget, cx)
        }));

        gtk::timeout_add(1000, clone!(clock => move || {
            clock.borrow_mut().update();
            Continue(true)
        }));

        clock.borrow_mut().update();

        clock
    }

    fn update(&mut self) {
        let now = time::now();
        let weekday = [ "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat" ][now.tm_wday as usize];

        self.text = format!("{} {} {:02}:{:02}", weekday, now.tm_mday, now.tm_hour, now.tm_min);
        self.widget.queue_draw();
    }

    fn draw(&self, widget: &gtk::DrawingArea, context: &cairo::Context) -> gtk::Inhibit {
        let width  = widget.get_allocated_width()  as f64;
        let height = widget.get_allocated_height() as f64;
        let text = &self.text;

        context.set_font_size(12.0);
        context.select_font_face("Droid Sans Mono",
                                 cairo::enums::FontSlant::Normal,
                                 cairo::enums::FontWeight::Normal);

        let extents = context.text_extents(text);
        let x = width  / 2.0 - extents.width  / 2.0 - extents.x_bearing;
        let y = height / 2.0 - extents.height / 2.0 - extents.y_bearing;

        context.move_to(x, y);
        context.set_source_rgba(1.0, 1.0, 1.0, 0.95);
        context.show_text(text);

        Inhibit(false)
    }
}

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

pub struct MemoryComponent {
    pub widget: gtk::DrawingArea,
    mem_usage: i64
}

impl MemoryComponent {
    pub fn new() -> Rc<RefCell<Self>> {
        let widget = gtk::DrawingArea::new();
        widget.set_size_request(50, -1);
        widget.set_vexpand(true);

        let memory_component = Rc::new(RefCell::new(MemoryComponent {
            widget: widget,
            mem_usage: 0
        }));

        memory_component.borrow_mut().update();

        {
            let ref mut widget = memory_component.borrow_mut().widget;

            // widget.connect_button_release_event(clone!(memory_component => move |widget, event| {
            //     memory_component.borrow().button_release(widget, event)
            // }));

            widget.connect_draw(clone!(memory_component => move |widget, cx| {
                memory_component.borrow().draw(widget, cx)
            }));
        }

        let (sx, rx) = channel::<bool>();

        gtk::timeout_add(5 * 1000, clone!(memory_component => move || {
            memory_component.borrow_mut().update();

            Continue(true)
        }));

        memory_component
    }

    fn update(&mut self) {
        let mut file = File::open("/proc/meminfo").expect("Couldn't open /proc/meminfo");
        let mut reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mem_total = i64::from_str(&lines.next().unwrap().unwrap().split_whitespace().nth(1).unwrap())
            .expect("Expected an integer from meminfo");

        lines.next(); // skip free

        let mem_available = i64::from_str(&lines.next().unwrap().unwrap().split_whitespace().nth(1).unwrap())
            .expect("Expected an integer from meminfo");

        self.mem_usage = 100 - (mem_available * 100 / mem_total);

        self.widget.queue_draw();
    }

    fn draw(&self, widget: &gtk::DrawingArea, context: &cairo::Context) -> gtk::Inhibit {
        let width  = widget.get_allocated_width()  as f64;
        let height = widget.get_allocated_height() as f64;
        let text = &format!("{}%", self.mem_usage);

        context.set_font_size(12.0);
        context.select_font_face("Droid Sans Mono",
                                 cairo::enums::FontSlant::Normal,
                                 cairo::enums::FontWeight::Normal);

        let extents = context.text_extents(text);
        let x = width  / 2.0 - extents.width  / 2.0 - extents.x_bearing;
        let y = height / 2.0 - extents.height / 2.0 - extents.y_bearing;

        match self.mem_usage {
             0...20 => context.set_source_rgba(0.2, 1.0, 0.5, 0.95),
            21...40 => context.set_source_rgba(0.1, 1.0, 0.1, 0.95),
            41...85 => context.set_source_rgba(1.0, 0.7, 0.0, 0.95),
            _       => context.set_source_rgba(1.0, 0.3, 0.1, 0.95),
        }

        context.move_to(x, y);
        context.show_text(text);

        Inhibit(false)
    }
}

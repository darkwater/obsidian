extern crate cairo;
extern crate gtk;
extern crate gdk;
extern crate gdk_sys;
extern crate time;

use gtk::prelude::*;
use std::cell::RefCell;
use std::fs::File;
use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::io::BufReader;
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::thread;

pub struct BatteryComponent {
    pub widget: gtk::DrawingArea,
    capacity: i8,
    charging: bool
}

impl BatteryComponent {
    pub fn new() -> Rc<RefCell<Self>> {
        let widget = gtk::DrawingArea::new();
        widget.set_size_request(50, -1);
        widget.set_vexpand(true);

        let battery_component = Rc::new(RefCell::new(BatteryComponent {
            widget: widget,
            capacity: 0,
            charging: true
        }));

        battery_component.borrow_mut().update();

        {
            let ref mut widget = battery_component.borrow_mut().widget;

            // widget.connect_button_release_event(clone!(battery_component => move |widget, event| {
            //     battery_component.borrow().button_release(widget, event)
            // }));

            widget.connect_draw(clone!(battery_component => move |widget, cx| {
                battery_component.borrow().draw(widget, cx)
            }));
        }

        let (sx, rx) = channel::<bool>();

        gtk::timeout_add(60 * 1000, clone!(battery_component => move || {
            battery_component.borrow_mut().update();

            Continue(true)
        }));

        gtk::timeout_add(50, clone!(battery_component => move || {
            if let Ok(charging) = rx.try_recv() {
                let mut comp = battery_component.borrow_mut();
                comp.charging = charging;
                comp.widget.queue_draw();
            }

            Continue(true)
        }));

        thread::spawn(move || {
            let acpi = UnixStream::connect("/var/run/acpid.socket").unwrap();
            let reader = BufReader::new(acpi);

            // XXX: Depends on 'L' to be the last itype sent
            for line in reader.lines() {
                let line = line.unwrap();
                let mut split = line.split(' ');
                let event = split.next().unwrap();

                if event == "ac_adapter" {
                    let charging = split.last().unwrap();
                    let charging = i8::from_str_radix(&charging, 10).unwrap();
                    let _ = sx.send(charging == 1);
                }
            }
        });

        battery_component
    }

    fn update(&mut self) {
        let mut file = File::open("/sys/class/power_supply/BAT1/capacity").expect("Couldn't find battery");
        let mut string = String::with_capacity(4);
        let _ = file.read_to_string(&mut string);

        self.capacity = i8::from_str_radix(&string.trim(), 10).expect("Expected an integer from battery capacity.");

        let mut file = File::open("/sys/class/power_supply/BAT1/status").expect("Couldn't find battery");
        let mut string = String::with_capacity(4);
        let _ = file.read_to_string(&mut string);

        self.charging = match string.trim() {
            "Charging" => true,
            "Full"     => true,
            _          => false
        };

        self.widget.queue_draw();
    }

    fn draw(&self, widget: &gtk::DrawingArea, context: &cairo::Context) -> gtk::Inhibit {
        let width  = widget.get_allocated_width()  as f64;
        let height = widget.get_allocated_height() as f64;
        let text = &format!("{}%", self.capacity);

        context.set_font_size(12.0);
        context.select_font_face("Droid Sans Mono",
                                 cairo::enums::FontSlant::Normal,
                                 cairo::enums::FontWeight::Normal);

        let extents = context.text_extents(text);
        let x = width  / 2.0 - extents.width  / 2.0 - extents.x_bearing;
        let y = height / 2.0 - extents.height / 2.0 - extents.y_bearing;

        if self.charging {
            context.set_source_rgba(0.1, 0.7, 1.0, 0.95);
        } else {
            match self.capacity {
                 0... 15 => context.set_source_rgba(1.0, 0.2, 0.0, 0.95),
                16... 40 => context.set_source_rgba(1.0, 0.7, 0.0, 0.95),
                41... 80 => context.set_source_rgba(0.1, 1.0, 0.1, 0.95),
                81...100 => context.set_source_rgba(0.2, 1.0, 0.5, 0.95),
                _        => context.set_source_rgba(1.0, 1.0, 1.0, 0.95),
            }
        }

        context.move_to(x, y);
        context.show_text(text);

        Inhibit(false)
    }
}

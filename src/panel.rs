extern crate gdk;
extern crate gdk_sys;
extern crate gtk;

use components::*;
use gtk::prelude::*;
use separator::{self, Separator};
use status::*;
use std::cell::RefCell;
use std::ops::Mul;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub struct Panel {
    expanded:           bool,
    expanded_position:  (i32, i32),
    expanded_size:      (i32, i32),
    collapsed_position: (i32, i32),
    collapsed_size:     (i32, i32),
    status_items:       Vec<Box<StatusItem>>,
    window:             gtk::Window
}

enum PanelMsg {
    ToggleExpand
}

impl Panel {
    pub fn new(items: Vec<&str>) -> Rc<RefCell<Self>> {
        let status_items: Vec<_> = items.iter().map(|item| {
            let item: Box<StatusItem> = match *item {
                "memory"  => Box::new(MemoryStatusItem),
                "load"    => Box::new(LoadStatusItem),
                "battery" => Box::new(BatteryStatusItem),
                "clock"   => Box::new(ClockStatusItem),
                _         => unreachable!()
            };

            item
        }).collect();

        if gtk::init().is_err() {
            panic!("Failed to initialize GTK.");
        }

        let window = gtk::Window::new(gtk::WindowType::Popup);
        window.set_name("panel");
        window.set_type_hint(gdk::WindowTypeHint::Dock);
        window.set_decorated(false);

        let screen = window.get_screen().unwrap();
        let monitor_id = screen.get_primary_monitor();
        let monitor = screen.get_monitor_geometry(monitor_id);

        window.set_app_paintable(true);
        let visual = screen.get_rgba_visual().unwrap();
        window.set_visual(Some(&visual));

        let collapsed_height = 75;
        let expanded_height  = 90;

        let collapsed_position = (monitor.x, monitor.y + monitor.height - 25);
        let collapsed_size     = (monitor.width, collapsed_height);

        let expanded_position = (monitor.x, monitor.y + monitor.height - expanded_height);
        let expanded_size     = (monitor.width, expanded_height);

        window.move_(collapsed_position.0, collapsed_position.1);
        window.resize(collapsed_size.0, collapsed_size.1);

        // reserve space
        // topw = window.get_toplevel().window
        // topw.property_change("_NET_WM_STRUT","CARDINAL",32,gtk.gdk.PROP_MODE_REPLACE,
        //                      [0, 0, bar_size, 0])
        // topw.property_change("_NET_WM_STRUT_PARTIAL","CARDINAL",32,gtk.gdk.PROP_MODE_REPLACE,
        //                      [0, 0, bar_size, 0, 0, 0, 0, 0, x, x+width, 0, 0])

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_homogeneous(true);
        window.add(&container);

        let top_bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.add(&top_bar);

        let middle_bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.add(&middle_bar);

        let bottom_bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.add(&bottom_bar);

        let workspaces = WorkspacesComponent::new();
        top_bar.add(&workspaces.borrow().widget);

        let separator = Separator::new(separator::Type::Spacer);
        top_bar.add(&separator.borrow().widget);

        let mut first = true;
        for item in &status_items {
            if !item.check_available() { continue }

            if first {
                first = false
            } else {
                let separator = Separator::new(separator::Type::Visual(1));
                top_bar.add(&separator.borrow().widget);
            }

            let status = StatusComponent::new(item);
            top_bar.add(&status.borrow().widget);
        }

        window.show_all();

        let panel = Rc::new(RefCell::new(Panel {
            expanded: false,
            expanded_position: expanded_position,
            expanded_size: expanded_size,
            collapsed_position: collapsed_position,
            collapsed_size: collapsed_size,
            status_items: status_items,
            window: window
        }));

        {
            let ref window = panel.borrow().window;

            window.connect_button_release_event(clone!(panel => move |_, event| {
                match event.get_button() {
                    3 => panel.borrow_mut().update(PanelMsg::ToggleExpand),
                    _ => ()
                }
                Inhibit(false)
            }));

            window.get_window().unwrap().set_background_rgba(&gdk::RGBA {
                red:   0x1d as f64 / 255.0,
                green: 0x1f as f64 / 255.0,
                blue:  0x21 as f64 / 255.0,
                alpha: 0xeb as f64 / 255.0
            });

            window.connect_delete_event(|_, _| {
                gtk::main_quit();
                Inhibit(false)
            });
        }

        panel
    }

    fn update(&mut self, msg: PanelMsg) {
        use self::PanelMsg::*;
        match msg {
            ToggleExpand => self.toggle_expand()
        }
    }

    fn toggle_expand(&mut self) {
        self.expanded = !self.expanded;

        let start_position = self.window.get_position();
        let start_size = self.window.get_size();

        let end_position;
        let end_size;

        match self.expanded {
            true  => { end_position = self.expanded_position;  end_size = self.expanded_size;  }
            false => { end_position = self.collapsed_position; end_size = self.collapsed_size; }
        }

        let transition_duration = Duration::from_millis(160);
        let transition_start    = Instant::now();

        #[inline]
        fn interp(start: i32, end: i32, i: f64) -> i32 {
            (start as f64 * (1.0 - i) + end as f64 * i) as i32
        }

        let ref window = self.window;
        gtk::timeout_add(5, clone!(window => move || {
            let mut transition_now = transition_start.elapsed().subsec_nanos() as f64
                                   / transition_duration.subsec_nanos() as f64;

            if transition_now > 1.0 { transition_now = 1.0 }

            let frame_position = (interp(start_position.0, end_position.0, transition_now),
                                  interp(start_position.1, end_position.1, transition_now));

            let frame_size = (interp(start_size.0, end_size.0, transition_now),
                              interp(start_size.1, end_size.1, transition_now));

            window.resize(frame_size.0, frame_size.1);
            window.move_(frame_position.0, frame_position.1);

            Continue(transition_now < 1.0)
        }));
    }
}

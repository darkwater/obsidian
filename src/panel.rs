extern crate gdk;
extern crate gdk_sys;
extern crate gtk;

use components::*;
use gtk::prelude::*;
use separator::{self, Separator};
use status::*;

pub struct Panel {
    expanded: bool,
    status_items: Vec<Box<StatusItem>>
}

impl Panel {
    pub fn new(items: Vec<&str>) -> Self {
        let items: Vec<_> = items.iter().map(|item| {
            let item: Box<StatusItem> = match *item {
                "memory"  => Box::new(MemoryStatusItem),
                "load"    => Box::new(LoadStatusItem),
                "battery" => Box::new(BatteryStatusItem),
                "clock"   => Box::new(ClockStatusItem),
                _         => unreachable!()
            };

            item
        }).collect();

        let mut panel = Panel {
            expanded: false,
            status_items: items
        };

        panel.init();

        panel
    }

    fn init(&mut self) {
        if gtk::init().is_err() {
            println!("Failed to initialize GTK.");
            return;
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

        let height = 25;

        window.move_(monitor.x, monitor.y + monitor.height - height);
        window.set_size_request(monitor.width, height);

        // reserve space
        // topw = window.get_toplevel().window
        // topw.property_change("_NET_WM_STRUT","CARDINAL",32,gtk.gdk.PROP_MODE_REPLACE,
        //                      [0, 0, bar_size, 0])
        // topw.property_change("_NET_WM_STRUT_PARTIAL","CARDINAL",32,gtk.gdk.PROP_MODE_REPLACE,
        //                      [0, 0, bar_size, 0, 0, 0, 0, 0, x, x+width, 0, 0])

        let grid = gtk::Grid::new();
        window.add(&grid);

        let workspaces = WorkspacesComponent::new();
        grid.add(&workspaces.borrow().widget);

        let separator = Separator::new(separator::Type::Spacer);
        grid.add(&separator.borrow().widget);

        let mut first = true;
        for item in &self.status_items {
            if !item.check_available() { continue }

            if first {
                first = false
            } else {
                let separator = Separator::new(separator::Type::Visual(1));
                grid.add(&separator.borrow().widget);
            }

            let status = StatusComponent::new(item);
            grid.add(&status.borrow().widget);
        }

        window.show_all();

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

    pub fn main(&self) {
        gtk::main();
    }
}

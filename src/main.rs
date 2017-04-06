#![feature(range_contains)]

extern crate gdk;
extern crate gdk_sys;
extern crate gtk;
extern crate time;

#[macro_use]
mod util;

mod components;
mod separator;
mod status;

use components::*;
use separator::*;
use gtk::prelude::*;

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = gtk::Window::new(gtk::WindowType::Popup);
    window.set_name("bar");
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
    let items = vec![ "memory", "load", "battery", "clock" ];
    for item in items {
        if first {
            first = false
        } else {
            let separator = Separator::new(separator::Type::Visual(1));
            grid.add(&separator.borrow().widget);
        }

        let update_fn = match item {
            "clock"   => status::clock,
            "battery" => status::battery,
            "load"    => status::load,
            "memory"  => status::memory,
            _ => unreachable!()
        };

        let status = StatusComponent::new(update_fn);
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

    gtk::main();
}

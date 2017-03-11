#![feature(range_contains)]

extern crate gtk;
extern crate gdk;

mod components;

use components::*;
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

    let style_context = window.get_style_context().unwrap();
    let css_provider = gtk::CssProvider::new();
    let _ = css_provider.load_from_data("* { background-color: #1d1f21; }");
    style_context.add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    let screen = window.get_screen().unwrap();
    let monitor_id = screen.get_primary_monitor();
    let monitor = screen.get_monitor_geometry(monitor_id);

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

    let workspaces = Workspaces::new();
    grid.add(&workspaces.borrow().widget);

    let separator = Separator::new(false, true);
    grid.add(&separator.borrow().widget);

    let clock = ClockComponent::new();
    grid.add(&clock.borrow().widget);

    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}

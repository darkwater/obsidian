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
mod panel;

use panel::Panel;

fn main() {
    let panel = Panel::new();

    gtk::main();
}

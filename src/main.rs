#![feature(inclusive_range, inclusive_range_syntax, range_contains)]

extern crate gdk;
extern crate gdk_sys;
extern crate gtk;
extern crate i3ipc;
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

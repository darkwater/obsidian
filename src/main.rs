#![feature(inclusive_range_syntax, range_contains)]

extern crate futures;
extern crate gdk;
extern crate gdk_sys;
extern crate gtk;
extern crate i3ipc;
extern crate leak;
extern crate time;

#[macro_use]
mod util;

mod components;
mod config;
mod panel;
mod separator;
mod status;

use config::Config;
use leak::Leak;
use panel::Panel;

fn main() {
    let config = Box::new(Config::default()).leak();

    let _panel = Panel::new(config);

    gtk::main();
}

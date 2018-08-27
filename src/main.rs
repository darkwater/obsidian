#![feature(nll, range_contains)]

#[macro_use] extern crate relm;
#[macro_use] extern crate relm_derive;
extern crate gdk;
extern crate gdk_sys;
extern crate glib;
extern crate gtk;
extern crate i3ipc;
extern crate relm_core;
extern crate time;

#[macro_use] mod util;
mod widgets;
mod config;
mod monitor;
mod panel;

use config::Config;
use panel::Panel;

fn main() {
    let config = Config::default();

    relm::run::<Panel>(config).unwrap();
}

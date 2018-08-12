#![feature(refcell_replace_swap, range_contains)]

#[macro_use] extern crate relm;
#[macro_use] extern crate relm_derive;
extern crate gdk;
extern crate gdk_sys;
extern crate glib;
extern crate gtk;
extern crate i3ipc;

extern crate time;

#[macro_use] mod util;
mod config;
mod panel;
mod components;

use config::Config;
use panel::Panel;

fn main() {
    let config = Config::default();

    relm::run::<Panel>(config).unwrap();
}

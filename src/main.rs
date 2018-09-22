#![feature(nll, range_contains)]

#[macro_use] extern crate relm;
#[macro_use] extern crate relm_derive;
#[macro_use] extern crate serde_derive;
extern crate gdk;
extern crate gdk_sys;
extern crate glib;
extern crate gtk;
extern crate i3ipc;
extern crate itertools;
extern crate relm_core;
extern crate serde;
extern crate time;

#[macro_use] mod util;
mod color;
mod config;
mod manager;
mod monitor;
mod bar;
mod status;
mod widgets;

fn main() {
    relm::run::<bar::Bar>(()).unwrap();
}

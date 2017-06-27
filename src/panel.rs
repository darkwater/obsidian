extern crate gdk;
extern crate gdk_sys;
extern crate glib;
extern crate gtk;

use components::*;
use config::Config;
use gtk::{Builder, Inhibit};
use gtk::prelude::*;
use relm::{Relm, RemoteRelm, Widget};
use self::glib::translate::ToGlibPtr;
use separator::{self, Separator};
use status::*;
use std::cell::{Cell, RefCell};
use std::process::Command;
use std::rc::Rc;

#[derive(Clone)]
pub struct PanelModel {
    config: &'static Config
}

#[derive(Msg)]
pub enum PanelMsg {
    Command(String),
    Quit,
}

#[derive(Clone)]
pub struct Panel {
    window:       gtk::Window,
    status_items: Rc<Vec<Box<StatusItem>>>,
}

impl Panel {
    fn run_cmd(&self, cmd: String) {
        let _ = Command::new("/bin/bash")
            .arg("-c")
            .arg(cmd)
            .spawn()
            .expect("failed to execute child");
    }
}

impl Widget for Panel {
    type Model = PanelModel;
    type ModelParam = Option<&'static Config>;
    type Msg = PanelMsg;
    type Root = gtk::Window;

    // Return the initial model.
    fn model(config: Self::ModelParam) -> Self::Model {
        Self::Model {
            config: config.unwrap()
        }
    }

    // Return the root of this widget.
    fn root(&self) -> &Self::Root {
        &self.window
    }

    fn view(relm: &RemoteRelm<Self>, model: &Self::Model) -> Self {
        let status_items: Vec<_> = model.config.status_items.iter().map(|item| {
            let item: Box<StatusItem> = match item.as_str() {
                "battery" => Box::new(BatteryStatusItem),
                "clock"   => Box::new(ClockStatusItem),
                "load"    => Box::new(LoadStatusItem),
                "memory"  => Box::new(MemoryStatusItem),
                "volume"  => Box::new(VolumeStatusItem),
                other     => panic!("unknown status component {:#?}", other)
            };

            item
        }).collect();

        let builder = Builder::new();
        let _ = builder.add_from_string(include_str!("panel.glade"));

        let window: gtk::Window = builder.get_object("window").unwrap();

        let screen = window.get_screen().unwrap();
        let monitor_id = screen.get_primary_monitor();
        let monitor = screen.get_monitor_geometry(monitor_id);

        let visual = screen.get_rgba_visual().unwrap();
        window.set_app_paintable(true);
        window.set_visual(Some(&visual));

        let height = 25;

        let (x, y) = (monitor.x, monitor.y + monitor.height - height);
        let (width, height) = (monitor.width, height);
        window.move_(x, y);
        window.resize(width, height);

        // reserve space
        // topw = window.get_toplevel().window
        // topw.property_change("_NET_WM_STRUT","CARDINAL",32,gtk.gdk.PROP_MODE_REPLACE,
        //                      [0, 0, bar_size, 0])
        // topw.property_change("_NET_WM_STRUT_PARTIAL","CARDINAL",32,gtk.gdk.PROP_MODE_REPLACE,
        //                      [0, 0, bar_size, 0, 0, 0, 0, 0, x, x+width, 0, 0])

        let cont_workspaces: gtk::Container = builder.get_object("container-workspaces").unwrap();
        let cont_music:      gtk::Container = builder.get_object("container-music").unwrap();
        let cont_status:     gtk::Container = builder.get_object("container-status").unwrap();

        let workspaces = WorkspacesComponent::new();
        cont_workspaces.add(&workspaces.borrow().widget);

        let music = MusicComponent::new(model.config);
        cont_music.add(&music.borrow().widget);

        let mut first = true;
        for item in &status_items {
            if item.check_available().is_err() { continue }

            if first {
                first = false
            } else {
                let separator = Separator::new();
                cont_status.add(&separator);
            }

            let status = StatusComponent::new(item, model.config);
            cont_status.add(&status.borrow().widget);
        }

        window.connect_realize(|window| {
            unsafe {
                let gdk_window = window.get_window().unwrap().to_glib_none().0;
                let type_ = gdk::Atom::intern("CARDINAL").to_glib_none().0;
                let mode = gdk_sys::GdkPropMode::Replace;

                let property = gdk::Atom::intern("_NET_WM_STRUT").to_glib_none().0;
                let strut = (&[ 0, 0, 0, 25u64 ]).as_ptr() as *const u8;
                gdk_sys::gdk_property_change(gdk_window, property, type_, 32, mode, strut, 4);

                let property = gdk::Atom::intern("_NET_WM_STRUT_PARTIAL").to_glib_none().0;
                let strut = (&[ 0, 0, 0, 25, 0, 0, 0, 0, 0, 0, 1920, 3840u64 ]).as_ptr() as *const u8;
                gdk_sys::gdk_property_change(gdk_window, property, type_, 32, mode, strut, 12);
            }
        });

        window.show_all();
        window.set_keep_above(true);

        let stream = relm.stream().clone();
        let config = model.config;
        window.connect_button_release_event(move |widget, event| {
            let width = widget.get_allocated_width() as f64;
            let (mouse_x, _mouse_y) = event.get_position();

            let command = match mouse_x / width * 3.0 {
                0.0...1.0 => config.launch.left.clone(),
                1.0...2.0 => config.launch.middle.clone(),
                2.0...3.0 => config.launch.right.clone(),
                _         => None
            }.map(PanelMsg::Command);

            match command {
                Some(cmd) => {
                    stream.emit(cmd);
                    Inhibit(true)
                },
                None => Inhibit(false)
            }
        });

        window.connect_draw(|widget, cx| {
            let width  = widget.get_allocated_width()  as f64;
            let height = widget.get_allocated_height() as f64;

            let (r, g, b, a) = (0.11, 0.12, 0.13, 0.92);
            cx.set_source_rgba(r, g, b, a);
            cx.rectangle(0.0, 0.0, width, height);
            cx.fill();

            Inhibit(false)
        });

        connect!(relm, window, connect_delete_event(_, _) (PanelMsg::Quit, Inhibit(false)));

        let panel = Panel {
            status_items: Rc::new(status_items),
            window:       window,
        };

        panel
    }

    fn update(&mut self, msg: Self::Msg, model: &mut Self::Model) {
        use self::PanelMsg::*;
        match msg {
            Command(cmd) => self.run_cmd(cmd),
            Quit         => gtk::main_quit(),
        }
    }
}

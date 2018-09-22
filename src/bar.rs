extern crate gdk;
extern crate gtk;
extern crate relm;

use config::Config;
use gdk::prelude::*;
use gtk::Inhibit;
use gtk::prelude::*;
use relm::{Component, ContainerWidget, EventStream, Relm, Update, Widget};

use ::widgets::workspace::WorkspaceWidget;
use ::widgets::monitor_bar::{MonitorBarMsg, MonitorBarWidget};
use ::manager::{Manager, ManagerMsg};
use ::monitor::Monitor;

pub struct BarModel {
    monitors: Vec<Box<dyn Monitor>>,
}

#[derive(Msg)]
pub enum BarMsg {
    Quit,
}

#[allow(unused)] // We must store Components to keep their channels
pub struct Bar {
    model:       BarModel,
    config:      &'static Config,
    window:      gtk::Window,
    workspaces:  Component<WorkspaceWidget>,
    manager:     EventStream<ManagerMsg>,
    bar_display: Component<MonitorBarWidget>,
}

impl Bar {
}

impl Update for Bar {
    type Model = BarModel;
    type ModelParam = ();
    type Msg = BarMsg;

    fn model(_: &Relm<Self>, _: Self::ModelParam) -> Self::Model {
        Self::Model {
            monitors: vec![],
        }
    }

    fn update(&mut self, msg: Self::Msg) {
        use self::BarMsg::*;
        match msg {
            Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Bar {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_wmclass("obsidian", "obsidian");
        window.set_title("obsidian");
        window.set_type_hint(gdk::WindowTypeHint::Dock);
        window.set_decorated(false);

        let screen     = window.get_screen().unwrap();
        let monitor_id = screen.get_primary_monitor();
        let monitor    = screen.get_monitor_geometry(monitor_id);

        let visual = screen.get_rgba_visual().unwrap();
        window.set_app_paintable(true);
        window.set_visual(Some(&visual));

        let mut config = Config::default();

        if config.dpi <= 0.0 {
            config.dpi = screen.get_resolution() / 96.0;
        }
        screen.set_resolution(96.0);

        let height = config.dpi_scale(25);

        let config = Box::new(config);
        let config = Box::leak(config);

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

        // window.connect_realize(|window| {
        //     unsafe {
        //         let gdk_window = window.get_window().unwrap().to_glib_none().0;
        //         let type_ = gdk::Atom::intern("CARDINAL").to_glib_none().0;
        //         let mode = gdk_sys::GdkPropMode::Replace;

        //         let property = gdk::Atom::intern("_NET_WM_STRUT").to_glib_none().0;
        //         let strut = (&[ 0, 0, 0, 25u64 ]).as_ptr() as *const u8;
        //         gdk_sys::gdk_property_change(gdk_window, property, type_, 32, mode, strut, 4);

        //         let property = gdk::Atom::intern("_NET_WM_STRUT_PARTIAL").to_glib_none().0;
        //         let strut = (&[ 0, 0, 0, 25, 0, 0, 0, 0, 0, 0, 1920, 3840u64 ]).as_ptr() as *const u8;
        //         gdk_sys::gdk_property_change(gdk_window, property, type_, 32, mode, strut, 12);
        //     }
        // });

        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.set_hexpand(true);
        container.set_vexpand(true);
        window.add(&container);

        let workspaces = container.add_widget::<WorkspaceWidget>(config);

        let manager          = relm::execute::<Manager>(config);
        let bar_display      = container.add_widget::<MonitorBarWidget>(config);
        // let popup_display = container.add_widget::<MonitorBarWidget>(config);
        connect_stream!(manager@ManagerMsg::DisplayUpdate(idx,    ref state), bar_display.stream(),   MonitorBarMsg::RecvUpdate(idx,   state.clone()));
        // connect_stream!(manager@ManagerMsg::DisplayUpdate(idx, ref state), popup_display.stream(), MonitorBarMsg::RecvUpdate(u));

        window.show_all();
        window.set_keep_above(true);

        window.connect_draw(|widget, cx| {
            let width  = widget.get_allocated_width()  as f64;
            let height = widget.get_allocated_height() as f64;

            let (r, g, b, a) = (0.11, 0.12, 0.13, 0.92);
            cx.set_source_rgba(r, g, b, a);
            cx.rectangle(0.0, 0.0, width, height);
            cx.fill();

            Inhibit(false)
        });

        connect!(relm, window, connect_delete_event(_, _), return (BarMsg::Quit, Inhibit(false)));

        Bar {
            model,
            config,
            window,
            workspaces,
            manager,
            bar_display,
        }
    }
}

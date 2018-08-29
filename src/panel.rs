extern crate gdk;
extern crate gtk;
extern crate relm;

use config::Config;
use gdk::prelude::*;
use gtk::Inhibit;
use gtk::prelude::*;
use relm::{Component, ContainerWidget, Relm, Update, Widget};

use ::widgets::workspace::WorkspaceWidget;
use ::widgets::monitor_bar::{MonitorBarMsg, MonitorBarWidget};
use ::manager::{Manager, ManagerMsg};
use ::monitor::Monitor;

pub struct PanelModel {
    config:   Config,
    monitors: Vec<Box<dyn Monitor>>,
}

#[derive(Msg)]
pub enum PanelMsg {
    Quit,
}

pub struct Panel {
    model:       PanelModel,
    window:      gtk::Window,
    workspaces:  Component<WorkspaceWidget>,
    bar_display: Component<MonitorBarWidget>,
}

impl Panel {
}

impl Update for Panel {
    type Model = PanelModel;
    type ModelParam = Config;
    type Msg = PanelMsg;

    fn model(_: &Relm<Self>, param: Self::ModelParam) -> Self::Model {
        Self::Model {
            config:   param,
            monitors: vec![],
        }
    }

    fn update(&mut self, msg: Self::Msg) {
        use self::PanelMsg::*;
        match msg {
            Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Panel {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, mut model: Self::Model) -> Self {
        let window = gtk::Window::new(gtk::WindowType::Toplevel);
        window.set_wmclass("obsidian", "obsidian");
        window.set_title("obsidian");
        window.set_type_hint(gdk::WindowTypeHint::Dock);
        window.set_decorated(false);

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

        let workspaces = container.add_widget::<WorkspaceWidget>(());

        let monitor_iface    = relm::execute::<::manager::Manager>(());
        let bar_display      = container.add_widget::<MonitorBarWidget>(());
        // let popup_display = container.add_widget::<MonitorPanelWidget>(());
        connect_stream!(monitor_iface@ManagerMsg::DisplayUpdate(idx, ref state), bar_display.stream(), MonitorBarMsg::RecvUpdate(idx, state.clone()));
        // connect_stream!(monitor_iface@ManagerMsg::DisplayUpdate(idx, ref state), popup_display.stream(), MonitorPanelMsg::RecvUpdate(u));

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

        connect!(relm, window, connect_delete_event(_, _), return (PanelMsg::Quit, Inhibit(false)));

        Panel {
            model,
            window,
            workspaces,
            bar_display,
        }
    }
}

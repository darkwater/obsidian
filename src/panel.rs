extern crate gdk;
extern crate gdk_sys;
extern crate glib;
extern crate gtk;

use components::*;
use config::Config;
use gtk::prelude::*;
use self::glib::translate::ToGlibPtr;
use separator::{self, Separator};
use status::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub struct Panel {
    opacity:           Rc<Cell<f64>>,
    status_items:      Vec<Box<StatusItem>>,
    window:            gtk::Window,
}

enum PanelMsg {
}

impl Panel {
    pub fn new(config: &'static Config) -> Rc<RefCell<Self>> {
        let items = vec![ "volume", "memory", "load", "battery", "clock" ];

        let status_items: Vec<_> = items.into_iter().map(|item| {
            let item: Box<StatusItem> = match item {
                "battery" => Box::new(BatteryStatusItem),
                "clock"   => Box::new(ClockStatusItem),
                "load"    => Box::new(LoadStatusItem),
                "memory"  => Box::new(MemoryStatusItem),
                "volume"  => Box::new(VolumeStatusItem),
                other     => panic!("unknown status component {:#?}", other)
            };

            item
        }).collect();

        if gtk::init().is_err() {
            panic!("Failed to initialize GTK.");
        }

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

        let container = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        window.add(&container);

        let workspaces = WorkspacesComponent::new();
        container.add(&workspaces.borrow().widget);

        // TODO: Properly center music widget
        let separator = Separator::new(separator::Type::Spacer);
        container.add(&separator.borrow().widget);

        let music = MusicComponent::new(config);
        container.add(&music.borrow().widget);

        let separator = Separator::new(separator::Type::Spacer);
        container.add(&separator.borrow().widget);

        let mut first = true;
        for item in &status_items {
            if item.check_available().is_err() { continue }

            if first {
                first = false
            } else {
                let separator = Separator::new(separator::Type::Visual(1));
                container.add(&separator.borrow().widget);
            }

            let status = StatusComponent::new(item, config);
            container.add(&status.borrow().widget);
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

        let panel = Rc::new(RefCell::new(Panel {
            opacity:           Rc::new(Cell::new(0.92)),
            status_items:      status_items,
            window:            window,
        }));

        {
            let ref window = panel.borrow().window;

            window.connect_button_release_event(clone!(panel => move |_, event| {
                match event.get_button() {
                    // 3 => panel.borrow_mut().update(PanelMsg::ToggleExpand),
                    _ => ()
                }
                Inhibit(false)
            }));

            window.connect_draw(clone!(panel => move |widget, cx| {
                let width  = widget.get_allocated_width()  as f64;
                let height = widget.get_allocated_height() as f64;

                let panel = panel.borrow();
                let (r, g, b, a) = (0.11, 0.12, 0.13, panel.opacity.get());
                cx.set_source_rgba(r, g, b, a);
                cx.rectangle(0.0, 0.0, width, height);
                cx.fill();

                Inhibit(false)
            }));

            window.connect_delete_event(|_, _| {
                gtk::main_quit();
                Inhibit(false)
            });
        }

        panel
    }

    fn update(&mut self, msg: PanelMsg) {
        use self::PanelMsg::*;
        match msg {
            // ToggleExpand => self.toggle_expand()
        }
    }
}

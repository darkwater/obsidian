extern crate gdk;
extern crate gdk_sys;
extern crate gtk;

use components::*;
use gtk::prelude::*;
use separator::{self, Separator};
use status::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

pub struct Panel {
    expanded:          bool,
    hidden:            bool,
    opacity:           Rc<Cell<f64>>,
    state_information: PanelStateInformation,
    status_items:      Vec<Box<StatusItem>>,
    window:            gtk::Window,
}

struct PanelStateInformation {
    collapsed_position: (i32, i32),
    collapsed_size:     (i32, i32),
    collapsed_opacity:  f64,
    expanded_position:  (i32, i32),
    expanded_size:      (i32, i32),
    expanded_opacity:   f64,
}

enum PanelMsg {
    ToggleExpand,
}

impl Panel {
    pub fn new() -> Rc<RefCell<Self>> {
        let items = vec![ "memory", "load", "battery", "clock" ];
        let extra_items = vec![ "volume" ];

        let status_items: Vec<_> = items.iter().map(|item| {
            let item: Box<StatusItem> = match *item {
                "battery" => Box::new(BatteryStatusItem),
                "clock"   => Box::new(ClockStatusItem),
                "load"    => Box::new(LoadStatusItem),
                "memory"  => Box::new(MemoryStatusItem),
                "volume"  => Box::new(VolumeStatusItem),
                _         => unreachable!()
            };

            item
        }).collect();

        let extra_status_items: Vec<_> = extra_items.iter().map(|item| {
            let item: Box<StatusItem> = match *item {
                "battery" => Box::new(BatteryStatusItem),
                "clock"   => Box::new(ClockStatusItem),
                "load"    => Box::new(LoadStatusItem),
                "memory"  => Box::new(MemoryStatusItem),
                "volume"  => Box::new(VolumeStatusItem),
                _         => unreachable!()
            };

            item
        }).collect();

        if gtk::init().is_err() {
            panic!("Failed to initialize GTK.");
        }

        let window = gtk::Window::new(gtk::WindowType::Popup);
        window.set_name("obsidian");
        window.set_type_hint(gdk::WindowTypeHint::Dock);
        window.set_decorated(false);

        let background_window = gtk::Window::new(gtk::WindowType::Popup);
        background_window.set_name("obsidian-background");
        background_window.set_type_hint(gdk::WindowTypeHint::Dock);
        background_window.set_decorated(false);

        let screen = window.get_screen().unwrap();
        let monitor_id = screen.get_primary_monitor();
        let monitor = screen.get_monitor_geometry(monitor_id);

        let visual = screen.get_rgba_visual().unwrap();
        window.set_app_paintable(true);
        window.set_visual(Some(&visual));
        background_window.set_app_paintable(true);
        background_window.set_visual(Some(&visual));

        let collapsed_visible_height = 25;
        let collapsed_height         = 75;
        let expanded_height          = 90;

        let state_information = PanelStateInformation {
            collapsed_position: (monitor.x, monitor.y + monitor.height - collapsed_visible_height),
            collapsed_size:     (monitor.width, collapsed_height),
            collapsed_opacity:  0.0,
            expanded_position:  (monitor.x, monitor.y + monitor.height - expanded_height),
            expanded_size:      (monitor.width, expanded_height),
            expanded_opacity:   0.92,
        };

        let ((x, y), (width, height)) = (state_information.collapsed_position, state_information.collapsed_size);
        window.move_(x, y);
        window.resize(width, height);
        background_window.move_(x, y);
        background_window.resize(width, height);

        // reserve space
        // topw = window.get_toplevel().window
        // topw.property_change("_NET_WM_STRUT","CARDINAL",32,gtk.gdk.PROP_MODE_REPLACE,
        //                      [0, 0, bar_size, 0])
        // topw.property_change("_NET_WM_STRUT_PARTIAL","CARDINAL",32,gtk.gdk.PROP_MODE_REPLACE,
        //                      [0, 0, bar_size, 0, 0, 0, 0, 0, x, x+width, 0, 0])

        let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        container.set_homogeneous(true);
        window.add(&container);

        let top_bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.add(&top_bar);

        let middle_bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.add(&middle_bar);

        let bottom_bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        container.add(&bottom_bar);

        let workspaces = WorkspacesComponent::new();
        top_bar.add(&workspaces.borrow().widget);

        let separator = Separator::new(separator::Type::Spacer);
        top_bar.add(&separator.borrow().widget);

        let mut first = true;
        for item in &status_items {
            if item.check_available().is_err() { continue }

            if first {
                first = false
            } else {
                let separator = Separator::new(separator::Type::Visual(1));
                top_bar.add(&separator.borrow().widget);
            }

            let status = StatusComponent::new(item);
            top_bar.add(&status.borrow().widget);
        }

        let separator = Separator::new(separator::Type::Spacer);
        middle_bar.add(&separator.borrow().widget);

        let mut first = true;
        for item in &extra_status_items {
            if item.check_available().is_err() { continue }

            if first {
                first = false
            } else {
                let separator = Separator::new(separator::Type::Visual(1));
                middle_bar.add(&separator.borrow().widget);
            }

            let status = StatusComponent::new(item);
            middle_bar.add(&status.borrow().widget);
        }

        background_window.show_all();
        background_window.set_keep_below(true);
        window.show_all();
        window.set_keep_above(true);

        let panel = Rc::new(RefCell::new(Panel {
            expanded:          false,
            hidden:            false,
            opacity:           Rc::new(Cell::new(state_information.collapsed_opacity)),
            state_information: state_information,
            status_items:      status_items,
            window:            window,
        }));

        background_window.connect_draw(move |widget, cx| {
            let width  = widget.get_allocated_width()  as f64;
            let height = widget.get_allocated_height() as f64;

            let (r, g, b, a) = (0.11, 0.12, 0.13, 0.92);
            cx.set_source_rgba(r, g, b, a);
            cx.rectangle(0.0, 0.0, width, height);
            cx.fill();

            Inhibit(true)
        });

        {
            let ref window = panel.borrow().window;

            window.connect_button_release_event(clone!(panel => move |_, event| {
                match event.get_button() {
                    3 => panel.borrow_mut().update(PanelMsg::ToggleExpand),
                    _ => ()
                }
                Inhibit(false)
            }));

            window.connect_draw(clone!(panel => move |widget, cx| {
                let panel = panel.borrow();

                // let window_position = widget.get_window().unwrap().get_position();
                // let collapsed_position = panel.state_information.collapsed_position;

                // let (x, y) = (collapsed_position.0 - window_position.0, collapsed_position.1 - window_position.1);
                // let (width, height) = panel.state_information.collapsed_size;
                // let (r, g, b, a) = (0.11, 0.12, 0.13, 0.92);
                // cx.set_source_rgba(r, g, b, a);
                // cx.rectangle(x as f64, y as f64, width as f64, height as f64);
                // cx.fill();

                let width  = widget.get_allocated_width()  as f64;
                let height = widget.get_allocated_height() as f64;

                let (r, g, b, a) = (0.10, 0.10, 0.10, panel.opacity.get());
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
            ToggleExpand => self.toggle_expand()
        }
    }

    fn toggle_expand(&mut self) {
        self.expanded = !self.expanded;

        let start_position = self.window.get_position();
        let start_size = self.window.get_size();
        let start_opacity;

        let end_position;
        let end_size;
        let end_opacity;

        let scale_factor = self.window.get_window().unwrap().get_scale_factor();
        let start_position = (start_position.0 / scale_factor, start_position.1 / scale_factor);

        match self.expanded {
            true => {
                start_opacity = self.state_information.collapsed_opacity;
                end_position  = self.state_information.expanded_position;
                end_size      = self.state_information.expanded_size;
                end_opacity   = self.state_information.expanded_opacity;
            }
            false => {
                start_opacity = self.state_information.expanded_opacity;
                end_position  = self.state_information.collapsed_position;
                end_size      = self.state_information.collapsed_size;
                end_opacity   = self.state_information.collapsed_opacity;
            }
        }

        let transition_duration = Duration::from_millis(160);
        let transition_start    = Instant::now();

        #[inline]
        fn interp(start: f64, end: f64, i: f64) -> f64 {
            (start * (1.0 - i) + end * i)
        }

        let window = self.window.clone();
        let opacity = self.opacity.clone();
        gtk::timeout_add(5, move || {
            let mut transition_now = transition_start.elapsed().subsec_nanos() as f64
                                   / transition_duration.subsec_nanos() as f64;

            if transition_now > 1.0 { transition_now = 1.0 }

            let frame_position = (interp(start_position.0 as f64, end_position.0 as f64, transition_now) as i32,
                                  interp(start_position.1 as f64, end_position.1 as f64, transition_now) as i32);

            let frame_size = (interp(start_size.0 as f64, end_size.0 as f64, transition_now) as i32,
                              interp(start_size.1 as f64, end_size.1 as f64, transition_now) as i32);

            let frame_opacity = interp(start_opacity, end_opacity, transition_now);

            window.resize(frame_size.0, frame_size.1);
            window.move_(frame_position.0, frame_position.1);
            opacity.set(frame_opacity);

            Continue(transition_now < 1.0)
        });
    }
}

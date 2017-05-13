extern crate cairo;
extern crate pango;
extern crate pangocairo;
extern crate gtk;
extern crate gdk;
extern crate gdk_sys;

use gtk::prelude::*;
use status::StatusItem;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

pub struct StatusComponent {
    pub widget: gtk::DrawingArea,
    text: String,
    icon: String,
    color: (f64, f64, f64, f64),
    size_request: Cell<SizeRequest>
}

pub enum StatusChange {
    Text(String),
    Icon(String),
    Color((f64, f64, f64, f64)),
    Size(SizeRequest)
}

#[derive(Copy, Clone)]
pub enum SizeRequest {
    Expand,
    Set,
    Keep
}

impl StatusComponent {
    pub fn new(status_item: &Box<StatusItem>) -> Rc<RefCell<Self>>
    {
        let widget = gtk::DrawingArea::new();
        widget.set_size_request(10, -1);
        widget.set_vexpand(true);

        let status_component = Rc::new(RefCell::new(StatusComponent {
            widget: widget,
            text: String::new(),
            icon: String::new(),
            color: (0.0, 0.0, 0.0, 0.0),
            size_request: Cell::new(SizeRequest::Keep)
        }));

        {
            let ref mut widget = status_component.borrow_mut().widget;

            // widget.connect_button_release_event(clone!(status_component => move |widget, event| {
            //     status_component.borrow().button_release(widget, event)
            // }));

            widget.connect_draw(clone!(status_component => move |widget, cx| {
                status_component.borrow().draw(widget, cx)
            }));
        }

        let (sx, rx) = mpsc::channel::<Vec<StatusChange>>();
        let update_fn = status_item.get_update_fun();

        thread::spawn(move || update_fn(sx));

        gtk::timeout_add(50, clone!(status_component => move || {
            if let Ok(changes) = rx.try_recv() {
                let mut comp = status_component.borrow_mut();

                for change in changes {
                    match change {
                        StatusChange::Text(text)   => comp.text  = text,
                        StatusChange::Icon(icon)   => comp.icon  = icon,
                        StatusChange::Color(color) => comp.color = color,
                        StatusChange::Size(req)    => comp.size_request.set(req),
                    }
                }

                comp.widget.queue_draw();
            }

            Continue(true)
        }));

        status_component
    }

    fn draw(&self, widget: &gtk::DrawingArea, context: &cairo::Context) -> gtk::Inhibit {
        let width   = widget.get_allocated_width()  as f64;
        let height  = widget.get_allocated_height() as f64;
        let margin  = 12.0; // around widget
        let padding = 10.0; // between icon and text

        let icon = &self.icon;
        let text = &self.text;

        let mut used_width = margin;

        if icon != "" {
            let font = pango::FontDescription::from_string("Material Icons 12");
            let layout = pangocairo::create_layout(context);
            layout.set_text(icon, icon.len() as i32);
            layout.set_font_description(Some(&font));

            let extents = layout.get_extents().0;
            let (icon_x, icon_y) = (extents.x as f64 / pango::SCALE as f64,
                                    extents.y as f64 / pango::SCALE as f64);
            let (icon_width, icon_height) = (extents.width  as f64 / pango::SCALE as f64,
                                             extents.height as f64 / pango::SCALE as f64);

            let x = -icon_x as f64 + margin;
            let y = -icon_y as f64 + height / 2.0 - icon_height / 2.0;

            let (r, g, b, a) = self.color;
            context.set_source_rgba(r, g, b, a);

            context.move_to(x, y);
            pangocairo::show_layout(&context, &layout);

            used_width += icon_width + padding;
        }

        context.set_font_size(12.0);
        context.select_font_face("Droid Sans Mono",
                                 cairo::enums::FontSlant::Normal,
                                 cairo::enums::FontWeight::Normal);

        let available_space = width - used_width - margin;
        let extents = context.text_extents(text);
        let x = used_width + available_space / 2.0 - extents.width  / 2.0 - extents.x_bearing;
        let y =              height          / 2.0 - extents.height / 2.0 - extents.y_bearing;

        let (r, g, b, a) = self.color;
        context.set_source_rgba(r, g, b, a);

        context.move_to(x, y);
        context.show_text(text);

        used_width += extents.x_advance + margin;

        match self.size_request.get() {
            SizeRequest::Expand => {
                if used_width > width {
                    widget.set_size_request(used_width as i32, -1);
                    self.size_request.set(SizeRequest::Keep);
                }
            },
            SizeRequest::Set => {
                widget.set_size_request(used_width as i32, -1);
                self.size_request.set(SizeRequest::Keep);
            },
            SizeRequest::Keep => {}
        }

        Inhibit(false)
    }
}

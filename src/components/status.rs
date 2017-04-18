extern crate cairo;
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
    color: (f64, f64, f64, f64),
    size_request: Cell<SizeRequest>
}

pub enum StatusChange {
    Text(String),
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
        let width  = widget.get_allocated_width()  as f64;
        let height = widget.get_allocated_height() as f64;
        let text = &self.text;

        context.set_font_size(12.0);
        context.select_font_face("Droid Sans Mono",
                                 cairo::enums::FontSlant::Normal,
                                 cairo::enums::FontWeight::Normal);

        let extents = context.text_extents(text);
        let x = width  / 2.0 - extents.width  / 2.0 - extents.x_bearing;
        let y = height / 2.0 - extents.height / 2.0 - extents.y_bearing;

        let (r, g, b, a) = self.color;
        context.set_source_rgba(r, g, b, a);

        context.move_to(x, y);
        context.show_text(text);

        let used_width = extents.width + 30.0;

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

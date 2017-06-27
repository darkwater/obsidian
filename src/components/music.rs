extern crate cairo;
extern crate gdk;
extern crate gdk_sys;
extern crate gtk;
extern crate mpd;
extern crate pango;
extern crate pangocairo;

use config::{Color, Config};
use gtk::prelude::*;
use self::mpd::Idle;
use std::cell::{Cell, RefCell};
use std::default::Default;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;

pub struct MusicComponent {
    pub widget: gtk::DrawingArea,
    text: String,
    icon: String,
    color: Color,
}

enum MpdMessage {
    Text(String),
    Color(Color),
}

impl MusicComponent {
    pub fn new(config: &'static Config) -> Rc<RefCell<Self>> {
        let widget = gtk::DrawingArea::new();
        widget.set_size_request(10, -1);
        widget.set_vexpand(true);

        let music_component = Rc::new(RefCell::new(MusicComponent {
            widget: widget,
            text: String::new(),
            icon: String::new(),
            color: Color::default(),
        }));

        {
            let ref mut widget = music_component.borrow_mut().widget;

            // widget.connect_button_release_event(clone!(music_component => move |widget, event| {
            //     music_component.borrow().button_release(widget, event)
            // }));

            widget.connect_draw(clone!(music_component => move |widget, cx| {
                music_component.borrow().draw(widget, cx)
            }));
        }

        let (sx, rx) = mpsc::channel::<Vec<MpdMessage>>();

        thread::spawn(move || {
            let mut conn = mpd::Client::connect((config.mpd.host.as_str(), config.mpd.port)).unwrap();
            loop {
                use self::mpd::State::*;
                match conn.status().unwrap().state {
                    Stop => {
                        let _ = sx.send(vec![MpdMessage::Text(String::new())]);
                    },
                    state => {
                        let song = conn.currentsong().unwrap().unwrap();
                        let text;

                        if let Some(artist) = song.tags.get("Artist") {
                            text = format!("{} - {}", artist, song.title.unwrap_or("<no title>".to_string()));
                        } else {
                            text = format!("{}", song.file);
                        }

                        let color = match state {
                            Play  => config.get_color("green"),
                            Pause => config.get_color("yellow"),
                            _     => unreachable!()
                        };

                        let _ = sx.send(vec![
                            MpdMessage::Text(text),
                            MpdMessage::Color(color)
                        ]);
                    },
                }

                let _ = conn.wait(&[mpd::Subsystem::Player]);
            }
        });

        gtk::timeout_add(50, clone!(music_component => move || {
            if let Ok(changes) = rx.try_recv() {
                let mut comp = music_component.borrow_mut();

                for change in changes {
                    match change {
                        MpdMessage::Text(text)   => comp.text  = text,
                        MpdMessage::Color(color) => comp.color = color,
                    }
                }

                comp.widget.queue_draw();
            }

            Continue(true)
        }));

        music_component
    }

    fn draw(&self, widget: &gtk::DrawingArea, context: &cairo::Context) -> gtk::Inhibit {
        let width    = widget.get_allocated_width()  as f64;
        let height   = widget.get_allocated_height() as f64;
        let margin   = 12.0; // around widget
        let padding  = 10.0; // between icon and text

        let icon = &self.icon;
        let text = &self.text;

        let mut used_width = margin;

        let layout = pangocairo::create_layout(context);

        if icon != "" {
            let font = pango::FontDescription::from_string("Material Icons 12");
            layout.set_text(icon, icon.len() as i32);
            layout.set_font_description(Some(&font));

            let extents = layout.get_extents().0;
            let (icon_x, icon_y) = (extents.x as f64 / pango::SCALE as f64,
                                    extents.y as f64 / pango::SCALE as f64);
            let (icon_width, icon_height) = (extents.width  as f64 / pango::SCALE as f64,
                                             extents.height as f64 / pango::SCALE as f64);

            let x = -icon_x as f64 + margin;
            let y = -icon_y as f64 + height / 2.0 - icon_height / 2.0;

            let Color(r, g, b, a) = self.color;
            context.set_source_rgba(r, g, b, a);

            context.move_to(x, y);
            pangocairo::show_layout(&context, &layout);

            used_width += icon_width + padding;
        }

        let font = pango::FontDescription::from_string("Droid Sans Mono 9");
        layout.set_text(text, text.len() as i32);
        layout.set_font_description(Some(&font));

        let available_space = width - used_width - margin;
        let extents = layout.get_extents().1;

        let (text_x, text_y) = (extents.x as f64 / pango::SCALE as f64,
                                extents.y as f64 / pango::SCALE as f64);
        let (text_width, text_height) = (extents.width  as f64 / pango::SCALE as f64,
                                         extents.height as f64 / pango::SCALE as f64);

        let x = used_width + available_space / 2.0 - text_width  / 2.0 - text_x;
        let y =                       height / 2.0 - text_height / 2.0 - text_y;

        let Color(r, g, b, a) = self.color;
        context.set_source_rgba(r, g, b, a);

        context.move_to(x, y);
        pangocairo::show_layout(&context, &layout);

        used_width += text_width + margin;

        widget.set_size_request(used_width as i32, -1);

        Inhibit(false)
    }
}

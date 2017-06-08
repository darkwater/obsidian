extern crate time;

use components::*;
use config::Config;
use status::StatusItem;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;

pub struct LoadStatusItem;
impl StatusItem for LoadStatusItem {
    fn check_available(&self) -> Result<(), &str> {
        Ok(())
    }

    fn get_update_fun(&self) -> fn(mpsc::Sender<Vec<StatusChange>>, &'static Config) {
        fn fun(sx: mpsc::Sender<Vec<StatusChange>>, config: &'static Config) {
            let changes = vec![
                StatusChange::Icon("equalizer".to_string()),
            ];

            let _ = sx.send(changes);

            loop {
                let mut file = File::open("/proc/loadavg").expect("Couldn't open /proc/loadavg");
                let mut string = String::with_capacity(32);
                let _ = file.read_to_string(&mut string);
                let mut split = string.split_whitespace();

                let loadavg = f64::from_str(&split.nth(1).unwrap()).expect("Expected a float from /proc/loadavg");

                let file = File::open("/proc/cpuinfo").expect("Couldn't open /proc/cpuinfo");
                let reader = BufReader::new(file);
                let mut num_processors = 0;
                for line in reader.lines() {
                    if line.expect("Error while reading /proc/cpuinfo").starts_with("processor") {
                        num_processors += 1;
                    }
                }

                let normalized_loadavg = loadavg / num_processors as f64;

                let text = format!("{:.2}", loadavg);

                let color = match normalized_loadavg {
                    0.0...0.1 => config.get_color("blue"),
                    0.1...0.4 => config.get_color("green"),
                    0.4...0.8 => config.get_color("yellow"),
                    _         => config.get_color("red"),
                };

                let changes = vec![
                    StatusChange::Text(text),
                    StatusChange::Color(color),
                    StatusChange::Size(SizeRequest::Expand),
                ];

                let _ = sx.send(changes);

                let sleep_time = ::std::time::Duration::from_secs(5);
                thread::sleep(sleep_time);
            }
        }

        fun
    }
}

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

pub struct MemoryStatusItem;
impl StatusItem for MemoryStatusItem {
    fn check_available(&self) -> Result<(), &str> {
        Ok(())
    }

    fn get_update_fun(&self) -> fn(mpsc::Sender<Vec<StatusChange>>, &'static Config) {
        fn fun(sx: mpsc::Sender<Vec<StatusChange>>, config: &'static Config) {
            let changes = vec![
                StatusChange::Icon("memory".to_string()),
            ];

            let _ = sx.send(changes);

            loop {
                let file = File::open("/proc/meminfo").expect("Couldn't open /proc/meminfo");
                let reader = BufReader::new(file);
                let mut lines = reader.lines();

                // XXX: Currently relies on the order of items in meminfo
                let mem_total = i64::from_str(&lines.next().unwrap().unwrap().split_whitespace().nth(1).unwrap())
                    .expect("Expected an integer from meminfo");

                lines.next(); // skip 'free'

                let mem_available = i64::from_str(&lines.next().unwrap().unwrap().split_whitespace().nth(1).unwrap())
                    .expect("Expected an integer from meminfo");

                let mem_usage = 100 - (mem_available * 100 / mem_total);

                let text = format!("{}%", mem_usage);

                let color = match mem_usage {
                    0...20 => config.get_color("blue"),
                   21...40 => config.get_color("green"),
                   41...85 => config.get_color("yellow"),
                   _       => config.get_color("red"),
                };

                let changes = vec![
                    StatusChange::Text(text),
                    StatusChange::Color(color),
                    StatusChange::Size(SizeRequest::Expand),
                ];

                let _ = sx.send(changes);

                let sleep_time = ::std::time::Duration::from_secs(8);
                thread::sleep(sleep_time);
            }
        }

        fun
    }
}

extern crate time;

use components::*;
use status::StatusItem;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;

pub struct MemoryStatusItem;
impl StatusItem for MemoryStatusItem {
    fn check_available(&self) -> bool {
        true
    }

    fn get_update_fun(&self) -> fn(mpsc::Sender<Vec<StatusChange>>) {
        fn fun(sx: mpsc::Sender<Vec<StatusChange>>) {
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
                    0...20 => (0.2, 1.0, 0.5, 0.95),
                   21...40 => (0.1, 1.0, 0.1, 0.95),
                   41...85 => (1.0, 0.7, 0.0, 0.95),
                   _       => (1.0, 0.3, 0.1, 0.95),
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

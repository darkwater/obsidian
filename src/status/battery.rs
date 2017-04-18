extern crate time;

use components::*;
use status::StatusItem;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;

pub struct BatteryStatusItem;
impl StatusItem for BatteryStatusItem {
    fn check_available(&self) -> bool {
        true
    }

    fn get_update_fun(&self) -> fn(mpsc::Sender<Vec<StatusChange>>) {
        fn fun(sx: mpsc::Sender<Vec<StatusChange>>) {
            let changes = vec![
                StatusChange::Text("100%".to_string()),
                StatusChange::Size(SizeRequest::Set)
            ];

            let _ = sx.send(changes);

            fn determine_color(capacity: i8, charging: bool) -> (f64, f64, f64, f64) {
                if charging {
                    (0.1, 0.7, 1.0, 0.95)
                } else {
                    match capacity {
                         0... 15 => (1.0, 0.2, 0.0, 0.95),
                        16... 40 => (1.0, 0.7, 0.0, 0.95),
                        41... 80 => (0.1, 1.0, 0.1, 0.95),
                        81...100 => (0.2, 1.0, 0.5, 0.95),
                        _        => (1.0, 1.0, 1.0, 0.95),
                    }
                }
            }

            enum BatteryChange {
                Capacity(i8),
                Charging(bool)
            }

            let (bat_sx, bat_rx) = mpsc::channel::<BatteryChange>();

            let _ = thread::Builder::new().name("battery/listen".into()).spawn(clone!(bat_sx => move || {
                let acpi = UnixStream::connect("/var/run/acpid.socket").unwrap();
                let reader = BufReader::new(acpi);

                for line in reader.lines() {
                    let line = line.unwrap();
                    let mut split = line.split(' ');
                    let event = split.next().unwrap();

                    if event == "ac_adapter" {
                        let charging = split.last().unwrap();
                        let charging = i8::from_str_radix(&charging, 10).unwrap();
                        let _ = bat_sx.send(BatteryChange::Charging(charging == 1));
                    }
                }
            }));

            let _ = thread::Builder::new().name("battery/update".into()).spawn(clone!(bat_sx => move || {
                loop {
                    let mut file = File::open("/sys/class/power_supply/BAT1/capacity").expect("Couldn't find battery");
                    let mut string = String::with_capacity(4);
                    let _ = file.read_to_string(&mut string);

                    let capacity = i8::from_str(&string.trim()).expect("Expected an integer from battery capacity.");

                    let mut file = File::open("/sys/class/power_supply/BAT1/status").expect("Couldn't find battery");
                    let mut string = String::with_capacity(16);
                    let _ = file.read_to_string(&mut string);

                    let charging = match string.trim() {
                        "Charging" => true,
                        "Full"     => true,
                        _          => false
                    };

                    let _ = bat_sx.send(BatteryChange::Capacity(capacity));
                    let _ = bat_sx.send(BatteryChange::Charging(charging));

                    let sleep_time = ::std::time::Duration::from_secs(40);
                    thread::sleep(sleep_time);
                }
            }));

            let mut capacity = 0i8;
            let mut charging = false;

            loop {
                if let Ok(change) = bat_rx.try_recv() {
                    match change {
                        BatteryChange::Capacity(new_cap) => {
                            capacity = new_cap;

                            let text = format!("{}%", capacity);
                            let color = determine_color(capacity, charging);

                            let changes = vec![
                                StatusChange::Text(text),
                                StatusChange::Color(color)
                            ];

                            let _ = sx.send(changes);
                        },
                        BatteryChange::Charging(new_charge) => {
                            charging = new_charge;

                            let color = determine_color(capacity, charging);

                            let changes = vec![
                                StatusChange::Color(color)
                            ];

                            let _ = sx.send(changes);
                        }
                    }
                }

                let sleep_time = ::std::time::Duration::from_millis(50);
                thread::sleep(sleep_time);
            }
        }

        fun
    }
}

// vim: fdm=syntax fdn=1
extern crate time;

use status_component::*;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::mpsc::{channel, Sender};
use std::thread;

pub fn clock(tx: Sender<Vec<StatusChange>>) {
    let changes = vec![
        StatusChange::Text("Mmm 00 00:00".to_string()),
        StatusChange::Size(SizeRequest::Set)
    ];

    let _ = tx.send(changes);

    loop {
        let now = time::now();
        let weekday = [ "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat" ][now.tm_wday as usize];

        let color = match now.tm_hour {
             0... 5 => (0.43, 0.53, 0.55, 1.0),
             6...11 => (0.49, 0.76, 0.81, 1.0),
            12...17 => (0.72, 0.84, 0.55, 1.0),
            18...23 => (0.88, 0.67, 0.36, 1.0),
            _       => (1.0,  1.0,  1.0,  1.0)
        };

        let text = format!("{} {} {:02}:{:02}", weekday, now.tm_mday, now.tm_hour, now.tm_min);

        let changes = vec![
            StatusChange::Text(text),
            StatusChange::Color(color)
        ];

        let _ = tx.send(changes);

        let sleep_time = ::std::time::Duration::new(59 - now.tm_sec as u64, 1000000000 - now.tm_nsec as u32);
        thread::sleep(sleep_time);
    }
}
pub fn battery(tx: Sender<Vec<StatusChange>>) {
    let changes = vec![
        StatusChange::Text("100%".to_string()),
        StatusChange::Size(SizeRequest::Set)
    ];

    let _ = tx.send(changes);

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

    let (bat_tx, bat_rx) = channel::<BatteryChange>();

    let _ = thread::Builder::new().name("battery/listen".into()).spawn(clone!(bat_tx => move || {
        let acpi = UnixStream::connect("/var/run/acpid.socket").unwrap();
        let reader = BufReader::new(acpi);

        for line in reader.lines() {
            let line = line.unwrap();
            let mut split = line.split(' ');
            let event = split.next().unwrap();

            if event == "ac_adapter" {
                let charging = split.last().unwrap();
                let charging = i8::from_str_radix(&charging, 10).unwrap();
                let _ = bat_tx.send(BatteryChange::Charging(charging == 1));
            }
        }
    }));

    let _ = thread::Builder::new().name("battery/update".into()).spawn(clone!(bat_tx => move || {
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

            let _ = bat_tx.send(BatteryChange::Capacity(capacity));
            let _ = bat_tx.send(BatteryChange::Charging(charging));

            let sleep_time = ::std::time::Duration::from_secs(40);
            thread::sleep(sleep_time);
        }
    }));

    let mut capacity = 0i8;
    let mut charging = false;

    loop {
        if let Ok(change) = bat_rx.try_recv() {
            match change {
                BatteryChange::Capacity(capacity) => {
                    let text = format!("{}%", capacity);
                    let color = determine_color(capacity, charging);

                    let changes = vec![
                        StatusChange::Text(text),
                        StatusChange::Color(color)
                    ];

                    let _ = tx.send(changes);
                },
                BatteryChange::Charging(charging) => {
                    let color = determine_color(capacity, charging);

                    let changes = vec![
                        StatusChange::Color(color)
                    ];

                    let _ = tx.send(changes);
                }
            }
        }

        let sleep_time = ::std::time::Duration::from_millis(50);
        thread::sleep(sleep_time);
    }
}

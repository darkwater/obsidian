extern crate time;

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::io::ErrorKind;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use config::Config;
use itertools::Itertools;
use relm_core::Sender;

use ::monitor::*;

pub struct Battery {
    syspath:  PathBuf,
    capacity: u8,
    charging: bool,
}

impl Default for Battery {
    fn default() -> Self {
        let mut bat = Battery {
            syspath:  PathBuf::from("/sys/class/power_supply/BAT0"),
            capacity: 0,
            charging: false,
        };

        bat.read_capacity();
        bat.read_charging();

        bat
    }
}

impl Battery {
    fn read_capacity(&mut self) {
        let mut f = File::open(self.syspath.join("capacity")).expect("failed to open BAT/capacity");
        let mut s = String::with_capacity(5);
        f.read_to_string(&mut s).expect("failed to read BAT/capacity");;

        self.capacity = u8::from_str_radix(s.trim(), 10).expect("invalid number from BAT/capacity");
    }

    fn read_charging(&mut self) {
        let mut f = File::open(self.syspath.join("status")).expect("failed to open BAT/status");
        let mut s = String::with_capacity(16);
        f.read_to_string(&mut s).expect("failed to read BAT/status");;

        self.charging = match s.trim() {
            "Charging" => true,
            "Full"     => true,
            _          => false
        };
    }

    fn send_state(&self, config: &'static Config, channel: &Sender<MonitorMsg>) {
        let color = if self.charging {
            config.get_color("blue")
        } else {
            match self.capacity {
                 0..= 15 => config.get_color("red"),
                16..= 40 => config.get_color("yellow"),
                41..=100 => config.get_color("green"),
                _        => panic!("battery capacity outside range 0..100")
            }
        };

        let relevance = match self.capacity < 40 {
            true  => Relevance::Urgent,
            false => Relevance::Background,
        };

        let text = format!("{} {}", self.charging, self.capacity);
        channel.send(MonitorMsg::SetText(text));
        channel.send(MonitorMsg::SetColor(color));
        channel.send(MonitorMsg::SetRelevance(Relevance::Urgent));
    }
}

impl Monitor for Battery {
    fn start(mut self, config: &'static Config, channel: Sender<MonitorMsg>) {
        self.send_state(config, &channel);

        let sock_timeout = 10;

        thread::spawn(move || {
            let acpi = UnixStream::connect("/var/run/acpid.socket").expect("couldn't open acpid socket");
            acpi.set_read_timeout(Some(Duration::from_secs(sock_timeout))).expect("failed to set timeout on acpid socket");
            let mut acpi = BufReader::new(acpi);
            let mut s = String::new();

            loop {
                loop {
                    match acpi.read_line(&mut s) {
                        Ok(_) => {
                            s.truncate(s.len() - 1);
                            let (event, _, _, value) = s.split(' ').collect_tuple().expect("unexpected output from acpid");
                            match event {
                                "ac_adapter" => {
                                    let value = usize::from_str_radix(value, 10).expect("unexpected output from acpid");
                                    self.charging = value != 0;
                                },
                                _ => (),
                            }
                            s.clear();

                            acpi.get_mut().set_read_timeout(Some(Duration::from_millis(200)))
                                .expect("failed to set timeout on acpid socket");
                        },
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                            acpi.get_mut().set_read_timeout(Some(Duration::from_secs(sock_timeout)))
                                .expect("failed to set timeout on acpid socket");

                            break;
                        },
                        Err(e) => panic!("couldn't read from acpid socket: {}", e),
                    }
                }

                self.read_capacity();

                self.send_state(config, &channel);
            }
        });
    }
}

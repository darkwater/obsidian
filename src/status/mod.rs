use components::status::StatusChange;
use config::Config;
use std::sync::mpsc;

pub trait StatusItem {
    fn check_available(&self) -> Result<(), &str>;
    fn get_update_fun(&self) -> fn(mpsc::Sender<Vec<StatusChange>>, &'static Config);
}

mod battery; pub use self::battery::BatteryStatusItem;
mod clock;   pub use self::clock::ClockStatusItem;
mod load;    pub use self::load::LoadStatusItem;
mod memory;  pub use self::memory::MemoryStatusItem;
mod volume;  pub use self::volume::VolumeStatusItem;

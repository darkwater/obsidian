use std::sync::mpsc;
use components::status::StatusChange;

pub trait StatusItem {
    fn check_available(&self) -> bool;
    fn get_update_fun(&self) -> fn(mpsc::Sender<Vec<StatusChange>>);
}

mod battery; pub use self::battery::BatteryStatusItem;
mod clock;   pub use self::clock::ClockStatusItem;
mod load;    pub use self::load::LoadStatusItem;
mod memory;  pub use self::memory::MemoryStatusItem;

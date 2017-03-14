#[macro_use]
mod util;

pub mod separator; pub use self::separator::Separator;
pub mod workspaces; pub use self::workspaces::WorkspacesComponent;
pub mod load; pub use self::load::LoadComponent;
pub mod memory; pub use self::memory::MemoryComponent;
pub mod battery; pub use self::battery::BatteryComponent;
pub mod clock; pub use self::clock::ClockComponent;

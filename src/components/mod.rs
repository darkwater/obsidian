#[macro_use]
mod util;

pub mod separator; pub use self::separator::Separator;
pub mod workspaces; pub use self::workspaces::WorkspacesComponent;
pub mod battery; pub use self::battery::BatteryComponent;
pub mod clock; pub use self::clock::ClockComponent;

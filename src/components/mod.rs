#[macro_use]
mod util;

mod separator; pub use self::separator::Separator;
mod workspaces; pub use self::workspaces::WorkspacesComponent;
mod battery; pub use self::battery::BatteryComponent;
mod clock; pub use self::clock::ClockComponent;

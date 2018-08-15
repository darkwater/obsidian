extern crate relm_core;

use std::thread;

#[no_mangle]
pub extern fn foo(sx: relm_core::Sender<String>) -> Result<f64, &'static str> {
    let mut c = 0;
    loop {
        c += 1;
        sx.send(format!("{}", c));
        thread::sleep_ms(1000);
    }
}

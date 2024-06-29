pub mod time_driver_systick;

pub fn init() {
    critical_section::with(|cs| time_driver_systick::init(cs));
}

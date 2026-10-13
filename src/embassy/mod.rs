//! Embassy clock porting interface.
//! Provides the embassy time tick source.
//! Users do not need to care about the internal interface of this module.

mod time_driver_systick;

pub fn init() {
    critical_section::with(time_driver_systick::init);
}

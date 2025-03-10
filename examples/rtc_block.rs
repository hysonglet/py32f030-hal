//! #  RTC demo
//!  该代码实现了一个RTC定时的功能
#![no_std]
#![no_main]

use hal::rtc::AnyRtc;
use py32f030_hal::{self as hal, mode::Blocking};

// use panic_halt as _;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());

    let rtc: AnyRtc<_, Blocking> = AnyRtc::new(p.RTC, Default::default()).unwrap();

    loop {
        defmt::info!("{}", rtc.read());
        rtc.wait_block(5);
    }
}

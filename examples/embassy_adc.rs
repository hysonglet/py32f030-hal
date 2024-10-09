#![no_std]
#![no_main]

use py32f030_hal::adc::{TemperatureChannel, VRrefChannel};
use py32f030_hal::mode::Async;
use py32f030_hal::{self as hal};

use embassy_executor::Spawner;
use embassy_time::Timer;
use hal::adc::{AnyAdc, ChannelConfig, Config};

// use panic_halt as _;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn run() {
    loop {
        Timer::after_secs(2).await;
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(Default::default());

    let adc: AnyAdc<_, Async> = AnyAdc::new(
        p.ADC,
        Config::default(),
        ChannelConfig::new_exclusive_single(),
        &[],
    )
    .unwrap();

    _spawner.spawn(run()).unwrap();

    loop {
        defmt::info!("temp {}", adc.read(TemperatureChannel).await,);
        defmt::info!("vref {}", adc.read(VRrefChannel).await);
        Timer::after_secs(2).await;
    }
}

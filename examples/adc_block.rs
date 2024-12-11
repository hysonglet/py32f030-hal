#![no_std]
#![no_main]

use hal::adc::{temperature, vrefence_internal, AdcChannel, AnyAdc, ChannelConfig, Config};
use hal::delay;
use py32f030_hal::adc::ConversionMode;
use py32f030_hal::{self as hal, mode::Blocking};

// use panic_halt as _;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());

    let adc: AnyAdc<_, Blocking> = AnyAdc::new(
        p.ADC,
        Config::default(),
        ChannelConfig::default()
            .over_write(false)
            .mode(ConversionMode::Discontinuous),
        &[AdcChannel::Channel11, AdcChannel::Channel12],
    )
    .unwrap();

    loop {
        adc.start();
        let temp = adc.read_block(1000000).unwrap();
        defmt::info!("temp: {}", temp);
        adc.start();
        let vol = adc.read_block(1000000).unwrap();

        defmt::info!(
            "temp: {}: {}, vol: {}: {}",
            temp,
            temperature(temp),
            vol,
            vrefence_internal(vol)
        );
        delay::delay_s(1);
    }
}

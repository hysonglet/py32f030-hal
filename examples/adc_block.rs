#![no_std]
#![no_main]

use defmt::Debug2Format;
use hal::adc::{temperature, AdcChannel, AnyAdc, ChannelConfig, Config};
use hal::delay;
use py32f030_hal::adc::{vrefence_internal, ConversionMode};
use py32f030_hal::{self as hal, mode::Blocking};

// use panic_halt as _;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());

    let adc: AnyAdc<_, Blocking> = AnyAdc::new(
        p.ADC,
        Config::default(),
        ChannelConfig::default().mode(ConversionMode::Discontinuous),
        &[AdcChannel::Channel11, AdcChannel::Channel12],
    )
    .unwrap();

    adc.start();
    loop {
        adc.start();
        let temp = adc.read_block(1000000).unwrap();
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
#![no_std]
#![no_main]

use hal::adc::{
    temperature, vrefence_internal, AdcChannel, AnyAdc, ChannelConfig, Config, SampleCycles,
    TrigleSignal,
};
use py32f030_hal::adc::ConversionMode;
use py32f030_hal::clock::sys_core_clock;
use py32f030_hal::{self as hal, mode::Blocking};

// use panic_halt as _;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());

    defmt::info!("{}", sys_core_clock());

    let adc: AnyAdc<_, Blocking> = AnyAdc::new(
        p.ADC,
        Config::default().sample(SampleCycles::Cycle_239_5),
        ChannelConfig::default()
            .over_write(false)
            .wait(true) // Wait for conversion to finish and reading to complete before starting the next conversion
            .singal(TrigleSignal::Soft)
            .mode(ConversionMode::Continuous),
        &[AdcChannel::Channel11, AdcChannel::Channel12],
    )
    .unwrap();

    adc.start();
    loop {
        // Just read in channel order.
        let temp = adc.read_block(1000000).unwrap();
        // adc.start();
        let vol = adc.read_block(1000000).unwrap();

        defmt::info!(
            "temp: {}: {}, vol: {}: {}",
            temp,
            temperature(temp),
            vol,
            vrefence_internal(vol)
        );
        hal::delay::delay_s(1);
    }
}

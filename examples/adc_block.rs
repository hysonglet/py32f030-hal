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
            .wait(true) // 转换完成后等待读取完毕再开始转换
            .singal(TrigleSignal::Soft)
            .mode(ConversionMode::Continuous),
        &[AdcChannel::Channel11, AdcChannel::Channel12],
    )
    .unwrap();

    adc.start();
    loop {
        // 按通道顺序读取即可
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

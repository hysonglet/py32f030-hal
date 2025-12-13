#![no_std]
#![no_main]

use core::cell::RefCell;

use cortex_m::interrupt::{self, Mutex};
use PY32f030xx_pac::{adc, ADC};

use defmt::info;
use hal::adc::{AdcChannel, AnyAdc, ChannelConfig, Config, SampleCycles, TrigleSignal};
use heapless::spsc::Queue;

use py32f030_hal::adc::{ConversionMode, Event};
use py32f030_hal::clock::sys_core_clock;
use py32f030_hal::interrupt::BindInterrupt;
use py32f030_hal::{self as hal, mode::Blocking};

use {defmt_rtt as _, panic_probe as _};

static ADC_INSTANCE: Mutex<RefCell<Option<AnyAdc<hal::mcu::peripherals::ADC, Blocking>>>> =
    Mutex::new(RefCell::new(None));

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());
    defmt::info!("{}", sys_core_clock());

    let mut adc: AnyAdc<_, Blocking> = AnyAdc::new(
        p.ADC,
        Config::default().sample(SampleCycles::Cycle_239_5),
        ChannelConfig::default()
            .over_write(false)
            .wait(true) // 转换完成后等待读取完毕再开始转换
            .singal(TrigleSignal::Soft)
            .mode(ConversionMode::Continuous),
        &[AdcChannel::Channel11, AdcChannel::Channel12],
        // &[AdcChannel::Channel11],
    )
    .unwrap();

    let _ = interrupt::free(|cs| {
        adc.event_config(Event::EOC, true);
        ADC_INSTANCE.borrow(cs).replace(Some(adc));

        let mut adc_bind = ADC_INSTANCE.borrow(cs).borrow_mut();
        let adc = adc_bind.as_mut().unwrap();
        let _ = adc.id().bind(&|| {
            interrupt::free(|cs| {
                let mut adc_bind = ADC_INSTANCE.borrow(cs).borrow_mut();
                let adc = adc_bind.as_mut().unwrap();
                let _ = adc.read_once();
            })
        });
        adc.id().enable();
        adc.start();
    });

    // 使用闭包的方式在中断中调用闭包处理函数
    // 兼顾友好型 api
    static mut ADC_QUEUE: Queue<u16, 128> = Queue::new();
    loop {
        cortex_m::asm::wfi();

        defmt::info!("adc value: {}", unsafe { ADC_QUEUE.dequeue().unwrap() });
    }
}

#![no_std]
#![no_main]

use core::cell::RefCell;
use cortex_m::interrupt::{self, Mutex};
use hal::adc::{AdcChannel, AnyAdc, ChannelConfig, Config, SampleCycles, TrigleSignal};
use heapless::spsc::Queue;

use py32f030_hal::adc::{ConversionMode, Event};
use py32f030_hal::clock::sys_core_clock;
use py32f030_hal::interrupt::BindInterrupt;
use py32f030_hal::{self as hal, mode::Blocking};

use {defmt_rtt as _, panic_probe as _};

static ADC_INSTANCE: Mutex<RefCell<Option<AnyAdc<hal::mcu::peripherals::ADC, Blocking>>>> =
    Mutex::new(RefCell::new(None));

type AdcQueue = Queue<u16, 128>;

static ADC_QUEUE: Mutex<RefCell<AdcQueue>> = Mutex::new(RefCell::new(AdcQueue::new()));

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
        let _ = adc.id().bind(&|cs| {
            let mut adc = ADC_INSTANCE.borrow(cs).borrow_mut();
            let mut queue = ADC_QUEUE.borrow(cs).borrow_mut();
            let _ = queue.enqueue(adc.as_mut().unwrap().read_once());
        });
        adc.id().enable_irq();
        adc.start();
    });

    loop {
        cortex_m::asm::wfi();

        interrupt::free(|cs| {
            let mut queue = ADC_QUEUE.borrow(cs).borrow_mut();
            while let Some(v) = queue.dequeue() {
                defmt::info!("adc value: {}", v);
            }
        })
    }
}

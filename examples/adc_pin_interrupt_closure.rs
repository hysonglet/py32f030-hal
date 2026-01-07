#![no_std]
#![no_main]

use core::cell::RefCell;
use cortex_m::interrupt::{self, Mutex};
use hal::adc::{AnyAdc, ChannelConfig, Config, ConversionMode, Event, SampleCycles, TrigleSignal};
use hal::clock::sys_core_clock;
use hal::interrupt::BindInterrupt;
use hal::mcu::peripherals::ADC;
use hal::mode::Blocking;
use heapless::spsc::Queue;
use py32f030_hal as hal;
use py32f030_hal::adc::AnalogPin;

use {defmt_rtt as _, panic_probe as _};

// 多线程中使用ADC，需要使用中断
static ADC_INSTANCE: Mutex<RefCell<Option<AnyAdc<ADC, Blocking>>>> = Mutex::new(RefCell::new(None));

type AdcQueue = Queue<u16, 128>;

static ADC_QUEUE: Mutex<RefCell<AdcQueue>> = Mutex::new(RefCell::new(AdcQueue::new()));

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = hal::init(Default::default());
    defmt::info!("{}", sys_core_clock());

    let gpioa = p.GPIOA.split();
    // gpio::Analog::new(gpioa.PA0);
    let channel_pin = gpioa.PA7;
    channel_pin.as_anlog();

    let mut adc: AnyAdc<_, Blocking> = AnyAdc::new(
        p.ADC,
        Config::default().sample(SampleCycles::Cycle_239_5),
        ChannelConfig::default()
            .over_write(false)
            .wait(true) // 转换完成后等待读取完毕再开始转换
            .singal(TrigleSignal::Soft)
            .mode(ConversionMode::Continuous),
        // 读
        &[channel_pin.channel()],
    )
    .unwrap();

    let _ = interrupt::free(|cs| {
        adc.event_config(Event::EOC, true);
        ADC_INSTANCE.borrow(cs).replace(Some(adc));
        let mut adc_bind = ADC_INSTANCE.borrow(cs).borrow_mut();
        let adc = adc_bind.as_mut().unwrap();

        static mut SUM: u64 = 0;
        adc.id()
            .bind(&|cs| {
                // 拿到ADC实例
                let mut adc_borrow = ADC_INSTANCE.borrow(cs).borrow_mut();
                // 获取队列的所有权
                let mut queue = ADC_QUEUE.borrow(cs).borrow_mut();
                let tmp = adc_borrow.as_mut().unwrap().read_once();
                let _ = queue.enqueue(tmp);
                unsafe {
                    SUM += tmp as u64;
                    // info!("{}", SUM);
                }
            })
            .unwrap();

        // 开启中断
        adc.id().enable_irq();
        // 开始转换
        adc.start();
    });

    loop {
        interrupt::free(|cs| {
            let mut queue = ADC_QUEUE.borrow(cs).borrow_mut();
            while queue.len() > 0 {
                defmt::info!(
                    "adc: {},  redunt: {}",
                    queue.dequeue().unwrap(),
                    queue.len()
                );
            }
        })
    }
}

#![no_std]
#![no_main]

use core::cell::RefCell;
use enumset::EnumSet;
use fugit::ExtU32;
use hal::interrupt::BindInterrupt;
use hal::timer::advanced_timer::AnyTimer;
use py32f030_hal::{
    self as hal,
    mcu::peripherals::TIM1,
    mode::Blocking,
    timer::{advanced_timer::Counter, advanced_timer::Event},
};
use {defmt_rtt as _, panic_probe as _};

use cortex_m::interrupt::{self, Mutex};

static COUNTER_INSTANCE: Mutex<RefCell<Option<Counter<TIM1, Blocking>>>> =
    Mutex::new(RefCell::new(None));

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("timer counter examples start...");
    let p = hal::init(Default::default());

    let timer = AnyTimer::<_, Blocking>::new(p.TIM1).unwrap();
    let mut counter = timer.as_counter();

    static mut CNT: u32 = 0;
    let _ = interrupt::free(|cs| {
        counter.enable_event(Event::UIF.into(), true);
        counter.clear_events(EnumSet::all());
        counter.id().enable_irq();
        counter.delay(1u32.secs());
        // 注册中断服务闭包处理逻辑
        let _ = counter.id().bind(&|cs| {
            let mut counter = COUNTER_INSTANCE.borrow(cs).borrow_mut();
            counter.as_mut().unwrap().clear_events(Event::UIF.into());
            unsafe {
                CNT += 1;
                defmt::info!("CNT: {}", CNT);
            }
        });
        COUNTER_INSTANCE.borrow(cs).replace(Some(counter));
    });

    // 延时 5s
    defmt::info!("repeat...");
    loop {
        cortex_m::asm::wfi();
        // defmt::info!("{}", unsafe { CNT });
    }
}

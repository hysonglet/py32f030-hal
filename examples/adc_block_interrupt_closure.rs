#![no_std]
#![no_main]

extern crate alloc;

use core::ptr::addr_of_mut;

use defmt::Debug2Format;
use hal::adc::{AdcChannel, AnyAdc, ChannelConfig, Config, Event, SampleCycles, TrigleSignal};

use py32f030_hal::adc::ConversionMode;
use py32f030_hal::clock::peripheral::PeripheralInterrupt;
use py32f030_hal::clock::sys_core_clock;
use py32f030_hal::{self as hal, mode::Blocking};

use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::entry]
fn main() -> ! {
    // -------- Setup Allocator --------
    const HEAP_SIZE: usize = 128;
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    #[global_allocator]
    static ALLOCATOR: alloc_cortex_m::CortexMHeap = alloc_cortex_m::CortexMHeap::empty();
    unsafe {
        #[allow(static_mut_refs)]
        ALLOCATOR.init(addr_of_mut!(HEAP) as usize, core::mem::size_of_val(&HEAP))
    }
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

    // 使用闭包的方式在中断中调用闭包处理函数
    // 兼顾友好型 api
    static mut QUEUE: [u16; 16] = [0; 16];
    // Ensure this example builds under compile configurations with embassy feature
    #[cfg(not(feature = "embassy"))]
    adc.on_interrupt(
        Event::EOC.into(), /* EOC 中断 */
        alloc::boxed::Box::new(move |adc| {
            /* 中断自动调用的闭包 */
            static mut CNT: usize = 0;
            unsafe {
                QUEUE[CNT] = adc;
                CNT += 1;
                if QUEUE.len() == CNT {
                    CNT = 0;
                }
            }

            // 打印转换成功的adc, 打印耗时会导致打印完毕后直接再次进入中断哦
            // defmt::info!("adc: {}", adc);
        }),
    );

    // 开启 EOC 中断
    adc.event_config(Event::EOC, true);
    adc.id().enable_interrupt();
    adc.start();
    loop {
        cortex_m::asm::wfi();

        defmt::info!(
            "adc {:?} sum: {} avrage: {}",
            Debug2Format(unsafe { &QUEUE }),
            unsafe { QUEUE.iter().sum::<u16>() },
            unsafe { QUEUE.iter().sum::<u16>() / QUEUE.len() as u16 }
        );
    }
}

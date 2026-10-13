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

use cortex_m_rt::ExceptionFrame;
use {defmt_rtt as _, panic_probe as _};

#[cortex_m_rt::exception]
unsafe fn HardFault(frame: &ExceptionFrame) -> ! {
    defmt::error!("HardFault pc={=u32}", frame.pc());
    loop {
        cortex_m::asm::wfi();
    }
}

#[no_mangle]
fn DefaultHandler() -> ! {
    defmt::error!("DefaultHandler: unhandled interrupt");
    loop {
        cortex_m::asm::wfi();
    }
}

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
            // Continuous interrupt-driven sampling: do NOT wait for a DR read between
            // conversions (wait:true stalls the stream after the first sample on this
            // part), and overwrite DR so an unconsumed value can't halt the ADC.
            .over_write(true)
            .wait(false)
            .singal(TrigleSignal::Soft)
            .mode(ConversionMode::Continuous),
        &[AdcChannel::Channel11, AdcChannel::Channel12],
        // &[AdcChannel::Channel11],
    )
    .unwrap();

    // Call the closure handler inside the interrupt using a closure.
    // Also provides a friendlier API.
    static mut QUEUE: [u16; 16] = [0; 16];
    static mut ISR_CNT: u32 = 0;
    static mut MAIN_CNT: u32 = 0;
    // Ensure this example builds under compile configurations with embassy feature
    #[cfg(not(feature = "embassy"))]
    adc.on_interrupt(
        Event::EOC.into(), /* EOC interrupt */
        alloc::boxed::Box::new(move |adc| {
            /* closure automatically invoked from the interrupt */
            static mut CNT: usize = 0;
            static mut STAGE: u8 = 0;
            unsafe {
                ISR_CNT = ISR_CNT.wrapping_add(1);
                if STAGE == 0 {
                    STAGE = 1;
                    defmt::info!("isr: entered adc={}", adc);
                }
                QUEUE[CNT] = adc;
                if STAGE == 1 {
                    STAGE = 2;
                    defmt::info!("isr: queued");
                }
                CNT += 1;
                if QUEUE.len() == CNT {
                    CNT = 0;
                }
            }

            // Printing the converted ADC value; note that printing takes time and may re-enter the interrupt immediately after.
            // defmt::info!("adc: {}", adc);
        }),
    );

    // Enable the EOC interrupt.
    adc.event_config(Event::EOC, true);
    adc.id().enable_interrupt();
    adc.start();
    defmt::info!("adc start ok");
    defmt::info!("adc entered loop");
    loop {
        unsafe {
            MAIN_CNT = MAIN_CNT.wrapping_add(1);
        }
        defmt::info!(
            "hb main={} isr={} queue0={}",
            unsafe { MAIN_CNT },
            unsafe { ISR_CNT },
            unsafe { QUEUE[0] }
        );
        cortex_m::asm::wfi();

        // Read via a raw pointer to avoid creating a shared reference to `static mut`.
        let queue = unsafe { &*core::ptr::addr_of!(QUEUE) };
        defmt::info!(
            "adc {:?} sum: {} avrage: {}",
            Debug2Format(queue),
            queue.iter().sum::<u16>(),
            queue.iter().sum::<u16>() / queue.len() as u16
        );
    }
}

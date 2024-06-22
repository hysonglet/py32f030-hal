#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use py32f030_hal as hal;
use hal::{InputPin, OutputPin, hal::digital::v2::ToggleableOutputPin, rcc::{HsiSys, clk_select, self, SysClk}};

const LOOP_CNT: u32 = 200000;

use hal::rcc::{self, clk_select::hsi};

#[cortex_m_rt::entry]
fn main() -> ! {
    let port = hal::gpio::GPIOA::Port;
    let p = port.split();

    let mut led = p.PA11.into_output();

    led.set_output_type(hal::gpio::PinOutputType::PushPull).unwrap();
    let _ = led.set_low().unwrap();
    let _ = led.set_high().unwrap();

    defmt::info!("led: {}", led.is_high().unwrap());

    let sysclk = SysClk::<HsiSys::<>>
    rcc::SysClk::<rcc::clk_select::hsi::HsiSys<rcc::clk_select::hsi::HsiHz::Hz8M, rcc::clk_select::hsi::HsiDiv::HsiDiv1>>::into(self);

    let clk = hal::rcc::Clk::take().unwrap();
    clk.sys_core_clk_config(hal::rcc::SysClkSource::HSISYS(HsiSys::default().set_clk(clk_select::HsiClk::Hz24M)));
    
    loop {
        for _ in 0..LOOP_CNT {
            cortex_m::asm::nop();
        }
        led.toggle().unwrap();
    }
}

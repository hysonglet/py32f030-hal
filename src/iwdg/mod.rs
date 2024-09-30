use crate::{clock, delay::wait_for_true_timeout_block};
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral};
mod hal;

pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

impl Instance for crate::mcu::peripherals::IWdg {}
impl hal::sealed::Instance for crate::mcu::peripherals::IWdg {}

pub struct IWdg<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

/* 重载的最大值 */
pub const RELOAD_MAX: u16 = 0xfff;
/* 内部看门狗时钟频率 */
const IWDG_CLOCK_HZ: u32 = 32768;

#[derive(Clone, Copy)]
pub enum Div {
    Div4 = 0,
    Div8 = 1,
    Div16 = 2,
    Div32 = 3,
    Div64 = 4,
    Div128 = 5,
    Div256 = 6,
}

#[derive(Clone, Copy)]
pub struct Config {
    div: Div,
    reload: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            div: Div::Div32,
            reload: 2048,
        }
    }
}

impl Config {
    pub fn timeout_us(&self) -> u32 {
        let hz = IWDG_CLOCK_HZ >> (self.div as usize + 2);
        1_000_000 * self.reload as u32 / hz
    }
}

impl<'d, T: Instance> IWdg<'d, T> {
    pub fn new(_iwdg: impl Peripheral<P = T>, config: Config) -> Self {
        into_ref!(_iwdg);

        /* 需要开启LSI时钟 */
        clock::lsi_enable().unwrap();

        T::start();

        // PeripheralClockIndex::.clock(true);
        T::enable_config();

        wait_for_true_timeout_block(100000, || !T::is_div_updating()).unwrap();
        wait_for_true_timeout_block(100000, || !T::is_reloading()).unwrap();
        T::set_div(config.div);
        T::set_reload(config.reload);
        Self { _t: PhantomData }
    }

    #[inline]
    pub fn start(&self) {
        T::feed();
        T::start();
    }

    #[inline]
    pub fn feed(&self) {
        T::feed();
    }
}

impl<'d, T: Instance> Drop for IWdg<'d, T> {
    fn drop(&mut self) {}
}

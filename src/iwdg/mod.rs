use crate::{clock, delay::wait_for_true_timeout_block};
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral};
mod instance;
mod types;

pub use types::*;

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + instance::sealed::Instance + 'static + Send {}

impl Instance for crate::mcu::peripherals::IWdg {}
impl instance::sealed::Instance for crate::mcu::peripherals::IWdg {}

pub struct IWdg<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance> IWdg<'d, T> {
    pub fn new(_iwdg: impl Peripheral<P = T>, config: Config) -> Self {
        into_ref!(_iwdg);

        // LSI clock must be enabled.
        clock::lsi_enable().unwrap();

        T::start();

        T::enable_config();

        wait_for_true_timeout_block(100000, || !T::is_div_updating()).unwrap();
        wait_for_true_timeout_block(100000, || !T::is_reloading()).unwrap();
        T::set_div(config.div);
        T::set_reload(config.reload);

        // Windowed watchdog.
        if config.window > 0 {
            wait_for_true_timeout_block(100000, || !T::is_window_updating()).unwrap();
            T::set_window(config.window);
        }
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

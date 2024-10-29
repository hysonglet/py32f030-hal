mod hal;
mod types;

use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral};

const FLASH_PAGE_SIZE: usize = 128;
const FLASH_SECTOR_SIZE: usize = 4 * 1024;
const FLASH_SECTOR_CNT: usize = 16;
const MAIN_FLASH_SIZE: usize = FLASH_SECTOR_SIZE * FLASH_SECTOR_CNT;

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}
impl Instance for crate::mcu::peripherals::FLASH {}
impl hal::sealed::Instance for crate::mcu::peripherals::FLASH {}

pub struct Flash<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance> Flash<'d, T> {
    pub fn new(_flash: impl Peripheral<P = T>) -> Self {
        into_ref!(_flash);

        Self { _t: PhantomData }
    }

    pub fn uuid() -> [u8; 16] {
        T::uuid()
    }
}

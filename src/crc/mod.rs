use crate::clock::peripheral::PeripheralClockIndex;
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral};
mod hal;

pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

impl Instance for crate::mcu::peripherals::CRC {}
impl hal::sealed::Instance for crate::mcu::peripherals::CRC {}

pub struct Crc<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance> Crc<'d, T> {
    pub fn new(_crc: impl Peripheral<P = T>) -> Self {
        into_ref!(_crc);

        PeripheralClockIndex::CRC.clock(true);
        T::reset();

        Self { _t: PhantomData }
    }

    pub fn accumulat(&self, buf: &[u32]) -> u32 {
        buf.iter().for_each(|v| T::write_data(*v));
        T::read_data()
    }

    pub fn calculate(&self, buf: &[u32]) -> u32 {
        T::reset();
        self.accumulat(buf)
    }

    pub fn reset(&self) {
        T::reset()
    }
}

impl<'d, T: Instance> Drop for Crc<'d, T> {
    fn drop(&mut self) {
        PeripheralClockIndex::CRC.clock(false);
    }
}

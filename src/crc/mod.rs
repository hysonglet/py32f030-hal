use crate::clock::peripheral::PeripheralClockIndex;
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral};

mod instance;

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + instance::sealed::Instance + 'static + Send {}

impl Instance for crate::mcu::peripherals::CRC {}
impl instance::sealed::Instance for crate::mcu::peripherals::CRC {}

pub struct Crc<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance> Crc<'d, T> {
    /// Create a new CRC instance.
    pub fn new(_crc: impl Peripheral<P = T>) -> Self {
        into_ref!(_crc);

        PeripheralClockIndex::CRC.clock(true);
        T::reset();

        Self { _t: PhantomData }
    }

    /// Accumulate and compute over multiple buffers.
    pub fn accumulat(&self, buf: &[u32]) -> u32 {
        buf.iter().for_each(|v| T::write_data(*v));
        T::read_data()
    }

    /// Compute the CRC of a whole buffer in a single call.
    pub fn calculate(&self, buf: &[u32]) -> u32 {
        T::reset();
        self.accumulat(buf)
    }

    /// Reset the result.
    pub fn reset(&self) {
        T::reset()
    }

    /// Write the independent data register (IDR), usable as scratch space.
    #[inline]
    pub fn set_idr(&self, v: u8) {
        T::set_idr(v);
    }

    /// Read the independent data register (IDR).
    #[inline]
    pub fn get_idr(&self) -> u8 {
        T::get_idr()
    }
}

impl<'d, T: Instance> Drop for Crc<'d, T> {
    fn drop(&mut self) {
        PeripheralClockIndex::CRC.clock(false);
    }
}

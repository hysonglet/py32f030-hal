use crate::clock::peripheral::PeripheralClockIndex;
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, Peripheral};

mod hal;

#[allow(private_bounds)]
pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

impl Instance for crate::mcu::peripherals::CRC {}
impl hal::sealed::Instance for crate::mcu::peripherals::CRC {}

pub struct Crc<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance> Crc<'d, T> {
    /// 新建CRC实例
    pub fn new(_crc: impl Peripheral<P = T>) -> Self {
        into_ref!(_crc);

        PeripheralClockIndex::CRC.clock(true);
        T::reset();

        Self { _t: PhantomData }
    }

    /// 多次积累计算
    pub fn accumulat(&self, buf: &[u32]) -> u32 {
        buf.iter().for_each(|v| T::write_data(*v));
        T::read_data()
    }

    /// 单词计算数组的crc值
    pub fn calculate(&self, buf: &[u32]) -> u32 {
        T::reset();
        self.accumulat(buf)
    }

    /// 复位结果
    pub fn reset(&self) {
        T::reset()
    }

    /// 写入独立数据寄存器（IDR），可用于暂存用户数据
    #[inline]
    pub fn set_idr(&self, v: u8) {
        T::set_idr(v);
    }

    /// 读取独立数据寄存器（IDR）
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

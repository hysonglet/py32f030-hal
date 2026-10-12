pub(super) mod sealed {
    // use super::super::*;
    use crate::pac;

    pub trait Instance {
        #[inline]
        fn block() -> &'static pac::crc::RegisterBlock {
            unsafe { pac::CRC::PTR.as_ref().unwrap() }
        }

        #[inline]
        fn reset() {
            Self::block().cr.write(|w| w.reset().set_bit())
        }

        fn read_data() -> u32 {
            Self::block().dr.read().bits()
        }

        fn write_data(v: u32) {
            Self::block().dr.write(|w| unsafe { w.dr().bits(v) })
        }

        /// 写入独立数据寄存器（IDR），可用于暂存用户数据
        #[inline]
        fn set_idr(v: u8) {
            Self::block().idr.write(|w| unsafe { w.idr().bits(v) })
        }

        /// 读取独立数据寄存器（IDR）
        #[inline]
        fn get_idr() -> u8 {
            Self::block().idr.read().idr().bits()
        }
    }
}

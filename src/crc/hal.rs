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
    }
}

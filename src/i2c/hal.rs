pub(super) mod sealed {
    use crate::clock::peripheral;
    use crate::pac;
    trait Instance {
        #[inline]
        fn block() -> &'static pac::i2c::RegisterBlock {
            unsafe { pac::I2C::PTR.as_ref().unwrap() }
        }

        #[inline]
        fn enable(en: bool) {
            peripheral::PeripheralClock::I2C.enable(en);
        }

        #[inline]
        fn reset() {
            peripheral::PeripheralClock::I2C.reset();
        }
    }
}

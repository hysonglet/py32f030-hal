pub(super) mod sealed {
    use super::super::*;
    use crate::pac;

    const FEED_KEY: u16 = 0xaaaa;
    const ACCESS_KEY: u16 = 0x5555;
    const START_KEY: u16 = 0xcccc;

    pub trait Instance {
        #[inline]
        fn block() -> &'static pac::iwdg::RegisterBlock {
            unsafe { pac::IWDG::PTR.as_ref().unwrap() }
        }

        #[inline]
        fn feed() {
            // Self::block()
            //     .kr
            //     .write(|w| unsafe { w.key().bits(FEED_KEY) })
            Self::block()
                .kr
                .write(|w| unsafe { w.bits(FEED_KEY as u32) })
        }

        #[inline]
        fn start() {
            // Self::block()
            //     .kr
            //     .write(|w| unsafe { w.key().bits(START_KEY) })
            Self::block()
                .kr
                .write(|w| unsafe { w.bits(START_KEY as u32) })
        }

        #[inline]
        fn enable_config() {
            // Self::block()
            //     .kr
            //     .write(|w| unsafe { w.key().bits(ACCESS_KEY) })
            Self::block()
                .kr
                .write(|w| unsafe { w.bits(ACCESS_KEY as u32) })
        }

        #[inline]
        fn set_div(div: Div) {
            Self::block()
                .pr
                .write(|w| unsafe { w.pr().bits(div as u8) })
        }

        #[inline]
        fn set_reload(load: u16) {
            assert!(load <= RELOAD_MAX);
            Self::block().rlr.write(|w| unsafe { w.rl().bits(load) });
        }

        #[inline]
        fn is_reloading() -> bool {
            Self::block().sr.read().rvu().bit()
        }

        #[inline]
        fn is_div_updating() -> bool {
            Self::block().sr.read().pvu().bit()
        }
    }
}

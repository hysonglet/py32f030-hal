pub mod sealed {
    use super::super::*;
    use crate::bit::*;
    use crate::clock::{self, ClockFrequency};
    use crate::pac;
    pub(crate) trait Instance {
        fn id() -> Id;

        #[inline]
        fn block() -> &'static pac::rtc::RegisterBlock {
            match Self::id() {
                Id::Rtc1 => unsafe { pac::RTC::PTR.as_ref().unwrap() },
            }
        }

        #[inline]
        fn configurable() -> bool {
            Self::block().crl.read().rtoff().bit()
        }

        #[inline]
        fn set_configurable(en: bool) {
            Self::block().crl.modify(|_, w| w.cnf().bit(en));
        }

        #[inline]
        fn enable_config() -> Result<(), Error> {
            pwr::rtc_unlock(true);
            // 等待可配置
            wait_for_true_timeout_block(100000, || Self::configurable())
                .map_err(|_| Error::Timeout)?;

            // 等待rtc 寄存器同步
            wait_for_true_timeout_block(100000, || Self::is_registers_synchronized())
                .map_err(|_| Error::Timeout)?;

            Self::set_configurable(true);

            Ok(())
        }

        #[inline]
        fn disable_config() {
            Self::set_configurable(false);
            pwr::rtc_unlock(false);
        }

        #[inline]
        fn is_registers_synchronized() -> bool {
            Self::block().crl.read().rsf().bit()
        }

        // #[inline]
        // fn clear_sync_flag() {
        //     Self::block().crl.modify(|_, w| w.rsf().clear_bit())
        // }

        /// fTR_CLK = fRTCCLK/(PRL[19:0]+1)
        #[inline]
        fn set_reload(val: u32) {
            let block = Self::block();

            let high = bit_mask_idx_get::<4>(16, val) as u8;
            let low = bit_mask_idx_get::<16>(0, val) as u16;
            block.prlh.write(|w| unsafe { w.prlh().bits(high) });
            block.prll.write(|w| unsafe { w.prll().bits(low) });
        }

        // #[inline]
        // fn get_div(div: u32) -> u32 {
        //     let block = Self::block();
        //     let high = block.divh.read().bits();
        //     let low = block.divl.read().bits();

        //     bit_mask_idx_modify::<16>(16, low, high)
        // }

        #[inline]
        fn get_counter() -> u32 {
            let block = Self::block();
            bit_mask_idx_modify::<16>(16, block.cntl.read().bits(), block.cnth.read().bits())
        }

        #[inline]
        fn set_counter(val: u32) {
            let block = Self::block();
            let high = bit_mask_idx_get::<16>(16, val);
            let low = bit_mask_idx_get::<16>(0, val);
            block.cnth.write(|w| unsafe { w.bits(high) });
            block.cntl.write(|w| unsafe { w.bits(low) });
        }

        #[inline]
        fn set_alarm(val: u32) {
            let block = Self::block();
            let high = bit_mask_idx_get::<16>(16, val);
            let low = bit_mask_idx_get::<16>(0, val);
            block.alrh.write(|w| unsafe { w.bits(high) });
            block.alrl.write(|w| unsafe { w.bits(low) });
        }
        // #[inline]
        // fn set_calibration(val: u8) {
        //     assert!(val < (1 << 7));

        //     Self::block()
        //         .rtccr
        //         .modify(|_, w| unsafe { w.cal().bits(val) });
        // }

        #[inline]
        fn set_clock(clock: RtcClock) -> Result<(), Error> {
            let div = match clock {
                RtcClock::LSI => {
                    clock::RtcClock::<clock::LSI>::config().map_err(|_| Error::Clock)?;
                    clock::RtcClock::<clock::LSI>::hz()
                }
                RtcClock::HSE_DIV_32 => {
                    clock::RtcClock::<clock::HSE>::config().map_err(|_| Error::Clock)?;
                    clock::RtcClock::<clock::HSE>::hz()
                }
                RtcClock::LSE => {
                    clock::RtcClock::<clock::LSE>::config().map_err(|_| Error::Clock)?;
                    clock::RtcClock::<clock::LSE>::hz()
                }
            };

            Self::set_reload(div - 1);
            Ok(())
        }

        #[inline]
        fn enable_interrupt(event: EventKind, en: bool) {
            let block = Self::block();
            match event {
                EventKind::Alarm => block.crh.modify(|_, w| w.alrie().bit(en)),
                EventKind::Second => block.crh.modify(|_, w| w.secie().bit(en)),
                EventKind::OverFlow => block.crh.modify(|_, w| w.owie().bit(en)),
            }
        }

        #[inline]
        fn is_enable_interrupt(event: EventKind) -> bool {
            let block = Self::block();
            match event {
                EventKind::Alarm => block.crh.read().alrie().bit(),
                EventKind::Second => block.crh.read().secie().bit(),
                EventKind::OverFlow => block.crh.read().owie().bit(),
            }
        }

        #[inline]
        fn is_interrupt(event: EventKind) -> bool {
            let block = Self::block();
            match event {
                EventKind::Alarm => block.crl.read().alrf().bit(),
                EventKind::Second => block.crl.read().secf().bit(),
                EventKind::OverFlow => block.crl.read().owf().bit(),
            }
        }

        #[inline]
        fn clear_interrupt(event: EventKind) {
            let block = Self::block();
            match event {
                EventKind::Alarm => block.crl.modify(|_, w| w.alrf().clear_bit()),
                EventKind::Second => block.crl.modify(|_, w| w.secf().clear_bit()),
                EventKind::OverFlow => block.crl.modify(|_, w| w.owf().clear_bit()),
            }
        }
    }
}

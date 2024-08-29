pub mod sealed {
    use super::super::*;
    use crate::clock::{self, ClockFrequency};
    use crate::common::BitOption;
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
            Self::block().crl.modify(|_, w| w.cnf().bit(en))
        }

        #[inline]
        fn is_registers_synchronized() -> bool {
            Self::block().crl.read().rsf().bit()
        }

        #[inline]
        fn clear_sync_flag() {
            Self::block().crl.modify(|_, w| w.rsf().clear_bit())
        }

        #[inline]
        fn is_overflow() -> bool {
            Self::block().crl.read().owf().bit()
        }

        #[inline]
        fn is_alarm() -> bool {
            Self::block().crl.read().alrf().bit()
        }

        #[inline]
        fn clear_alarm() {
            Self::block().crl.modify(|_, w| w.alrf().clear_bit())
        }

        #[inline]
        fn second_flag() -> bool {
            Self::block().crl.read().secf().bit()
        }

        #[inline]
        fn clear_second_flag() {
            Self::block().crl.modify(|_, w| w.secf().clear_bit())
        }

        /// fTR_CLK = fRTCCLK/(PRL[19:0]+1)
        #[inline]
        fn set_reload(val: u32) {
            let block = Self::block();
            let high: u8;
            let low: u16;

            high = BitOption::bit_mask_idx_get::<4>(16, val) as u8;
            low = BitOption::bit_mask_idx_get::<16>(0, val) as u16;
            block.prlh.write(|w| unsafe { w.prlh().bits(high) });
            block.prll.write(|w| unsafe { w.prll().bits(low) });
        }

        #[inline]
        fn get_div(div: u32) -> u32 {
            let block = Self::block();
            let high = block.divh.read().bits();
            let low = block.divl.read().bits();

            BitOption::bit_mask_idx_modify::<16>(16, low, high)
        }

        #[inline]
        fn get_counter() -> u32 {
            let block = Self::block();
            BitOption::bit_mask_idx_modify::<16>(
                16,
                block.cntl.read().bits(),
                block.cnth.read().bits(),
            )
        }
        #[inline]
        fn set_counter(val: u32) {
            let block = Self::block();
            let high = BitOption::bit_mask_idx_get::<16>(16, val);
            let low = BitOption::bit_mask_idx_get::<16>(0, val);
            block.cnth.write(|w| unsafe { w.bits(high) });
            block.cntl.write(|w| unsafe { w.bits(low) });
        }

        #[inline]
        fn set_alarm(val: u32) {
            let block = Self::block();
            let high = BitOption::bit_mask_idx_get::<16>(16, val);
            let low = BitOption::bit_mask_idx_get::<16>(0, val);
            block.alrh.write(|w| unsafe { w.bits(high) });
            block.alrl.write(|w| unsafe { w.bits(low) });
        }
        #[inline]
        fn set_calibration(val: u8) {
            assert!(val < (1 << 7));

            Self::block()
                .rtccr
                .modify(|_, w| unsafe { w.cal().bits(val) });
        }

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

        fn enable_second_interrupt(en: bool) {
            Self::block().crh.modify(|_, w| w.secie().bit(en))
        }

        fn enable_alarm_interrupt(en: bool) {
            Self::block().crh.modify(|_, w| w.alrie().bit(en))
        }
    }
}

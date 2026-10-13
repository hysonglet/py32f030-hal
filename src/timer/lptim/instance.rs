use super::types::*;
use super::LpTimer;

pub(crate) mod sealed {
    use super::*;
    use crate::pac::lptim::RegisterBlock;

    pub trait Instance {
        /// Timer index.
        fn id() -> LpTimer;

        #[inline]
        fn block() -> &'static RegisterBlock {
            unsafe { &*(crate::pac::LPTIM::ptr()) }
        }

        /// Select the core clock source (valid only when LPTIM is disabled; writes RCC_CCIPR.LPTIMSEL).
        #[inline]
        fn set_clock_source(src: ClockSource) {
            let rcc = unsafe { crate::pac::RCC::PTR.as_ref().unwrap() };
            rcc.ccipr
                .modify(|_, w| unsafe { w.lptim1sel().bits(src as u8) });
        }

        /// Enable / disable the LPTIM core.
        #[inline]
        fn enable_kernel(en: bool) {
            Self::block().cr.modify(|_, w| w.enable().bit(en));
        }

        /// Start a single count (software trigger): set ENABLE and write SNGSTRT.
        #[inline]
        fn start_single() {
            let b = Self::block();
            b.cr.modify(|_, w| w.enable().set_bit());
            b.cr.modify(|_, w| w.sngstrt().set_bit());
        }

        /// Start continuous counting (auto-reload and keep running after reaching ARR).
        #[inline]
        fn start_continuous() {
            Self::block()
                .cr
                .modify(|_, w| w.sngstrt().clear_bit().enable().set_bit());
        }

        /// Set the prescaler (valid only when LPTIM is disabled).
        #[inline]
        fn set_prescaler(p: Prescaler) {
            Self::block()
                .cfgr
                .modify(|_, w| unsafe { w.presc().bits(p as u8) });
        }

        /// Set the ARR preload enable (valid only when LPTIM is disabled).
        #[inline]
        fn set_preload(en: bool) {
            Self::block().cfgr.modify(|_, w| w.preload().bit(en));
        }

        /// Write the auto-reload register.
        #[inline]
        fn set_auto_reload(arr: u16) {
            Self::block().arr.write(|w| unsafe { w.arr().bits(arr) });
        }

        /// Read the auto-reload register.
        #[inline]
        fn get_auto_reload() -> u16 {
            Self::block().arr.read().arr().bits()
        }

        /// Read the current counter value.
        #[inline]
        fn counter() -> u16 {
            Self::block().cnt.read().cnt().bits()
        }

        /// Stop counting (disable the core).
        #[inline]
        fn stop() {
            Self::enable_kernel(false);
        }

        /// Enable / disable an interrupt event.
        #[inline]
        fn event_config(event: Event, en: bool) {
            match event {
                Event::AutoreloadMatch => {
                    Self::block().ier.modify(|_, w| w.arrmie().bit(en))
                }
            }
        }

        /// Query the event flag.
        #[inline]
        fn event_flag(event: Event) -> bool {
            match event {
                Event::AutoreloadMatch => Self::block().isr.read().arrm().bit_is_set(),
            }
        }

        /// Clear the event flag.
        #[inline]
        fn event_clear(event: Event) {
            match event {
                Event::AutoreloadMatch => {
                    Self::block().icr.write(|w| w.arrmcf().set_bit())
                }
            }
        }
    }
}

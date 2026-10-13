pub(super) mod sealed {
    use super::super::*;

    /// COMP instance register operations.
    ///
    /// COMP1 and COMP2 have two distinct register-block types in the PAC (comp1 /
    /// comp2) and cannot share a single `block()`, so the implementation is generated
    /// per instance via a macro.
    ///
    /// Note: named `CompInstance` rather than `Instance` to avoid ambiguity with
    /// `comp::Instance` (the public trait).
    pub trait CompInstance {
        fn id() -> Id;

        /// Enable/disable the comparator.
        fn enable(en: bool);
        /// Read the comparator output.
        fn output() -> bool;
        /// Apply the full configuration.
        fn config(config: Config);

        fn set_hysteresis(hyst: Hysteresis);
        fn set_power_mode(mode: PowerMode);
        fn set_polarity(polarity: Polarity);
        fn set_window_mode(en: bool);
        /// Lock the configuration (once LOCK is set, CSR is read-only until reset).
        fn lock();
        fn is_locked() -> bool;
        /// Digital filter.
        fn set_filter(en: bool, count: u8);

        /// Write hysteresis (COMP1 only; COMP2 has no such field, defaults to no-op).
        #[inline]
        fn write_hyst(_: Hysteresis) {}
        /// Write the scaler enable (COMP1 only; COMP2 has no such field, defaults to no-op).
        #[inline]
        fn write_scaler(_: bool) {}
    }

    macro_rules! impl_comp_instance {
        ($periph:ident, $pac:ident, $id:ident, $extras:ident) => {
            impl $crate::comp::instance::sealed::CompInstance for $crate::mcu::peripherals::$periph {
                #[inline]
                fn id() -> Id {
                    Id::$id
                }

                #[inline]
                fn enable(en: bool) {
                    unsafe { &*$crate::pac::$pac::PTR }
                        .csr
                        .modify(|_, w| w.comp_en().bit(en));
                }

                #[inline]
                fn output() -> bool {
                    unsafe { &*$crate::pac::$pac::PTR }
                        .csr
                        .read()
                        .comp_out()
                        .bit()
                }

                fn config(config: Config) {
                    // Disable the comparator before configuring.
                    Self::enable(false);

                    unsafe { &*$crate::pac::$pac::PTR }.csr.modify(|_, w| unsafe {
                        w.pwrmode()
                            .bits(config.power_mode as u8)
                            .polarity()
                            .bit(config.polarity == Polarity::Inverted)
                            .inpsel()
                            .bits(config.inpsel)
                            .inmsel()
                            .bits(config.inmsel)
                            .winmode()
                            .bit(config.window_mode)
                    });

                    Self::write_hyst(config.hysteresis);
                    Self::write_scaler(config.scaler_en);
                    Self::set_filter(config.filter_enable, config.filter_count);
                }

                #[inline]
                fn set_hysteresis(hyst: Hysteresis) {
                    Self::write_hyst(hyst);
                }

                #[inline]
                fn set_power_mode(mode: PowerMode) {
                    unsafe { &*$crate::pac::$pac::PTR }
                        .csr
                        .modify(|_, w| unsafe { w.pwrmode().bits(mode as u8) });
                }

                #[inline]
                fn set_polarity(polarity: Polarity) {
                    unsafe { &*$crate::pac::$pac::PTR }
                        .csr
                        .modify(|_, w| w.polarity().bit(polarity == Polarity::Inverted));
                }

                #[inline]
                fn set_window_mode(en: bool) {
                    unsafe { &*$crate::pac::$pac::PTR }
                        .csr
                        .modify(|_, w| w.winmode().bit(en));
                }

                #[inline]
                fn lock() {
                    unsafe { &*$crate::pac::$pac::PTR }
                        .csr
                        .modify(|_, w| w.lock().set_bit());
                }

                #[inline]
                fn is_locked() -> bool {
                    unsafe { &*$crate::pac::$pac::PTR }
                        .csr
                        .read()
                        .lock()
                        .bit()
                }

                #[inline]
                fn set_filter(en: bool, count: u8) {
                    unsafe { &*$crate::pac::$pac::PTR }
                        .fr
                        .modify(|_, w| unsafe { w.flten().bit(en).fltcnt().bits(count as u16) });
                }

                #[inline]
                fn write_hyst(hyst: Hysteresis) {
                    $crate::comp::instance::sealed::impl_comp_instance!(@write_hyst $extras, $pac, hyst);
                }

                #[inline]
                fn write_scaler(en: bool) {
                    $crate::comp::instance::sealed::impl_comp_instance!(@write_scaler $extras, $pac, en);
                }
            }

            impl $crate::comp::Instance for $crate::mcu::peripherals::$periph {}
        };

        // COMP1 only: hysteresis and scaler enable.
        (@write_hyst has_hyst, $pac:ident, $hyst:ident) => {
            unsafe { &*$crate::pac::$pac::PTR }
                .csr
                .modify(|_, w| w.hyst().bit($hyst as u8 != 0));
        };
        (@write_hyst no_hyst, $pac:ident, $hyst:ident) => {
            let _ = $hyst;
        };
        (@write_scaler has_hyst, $pac:ident, $en:ident) => {
            unsafe { &*$crate::pac::$pac::PTR }
                .csr
                .modify(|_, w| w.scaler_en().bit($en));
        };
        (@write_scaler no_hyst, $pac:ident, $en:ident) => {
            let _ = $en;
        };
    }

    pub(crate) use impl_comp_instance;
}

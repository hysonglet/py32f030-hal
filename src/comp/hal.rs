pub(super) mod sealed {
    use super::super::*;

    /// COMP 实例寄存器操作
    ///
    /// COMP1 与 COMP2 的寄存器块是 PAC 中两个不同类型（comp1 / comp2），
    /// 无法用统一的 `block()` 返回，故用宏按实例生成实现。
    ///
    /// 注意：这里命名为 `CompInstance` 而非 `Instance`，避免与
    /// `comp::Instance`（对外公开 trait）重名造成解析歧义。
    pub trait CompInstance {
        fn id() -> Id;

        /// 使能/关闭比较器
        fn enable(en: bool);
        /// 读取比较器输出
        fn output() -> bool;
        /// 应用完整配置
        fn config(config: Config);

        fn set_hysteresis(hyst: Hysteresis);
        fn set_power_mode(mode: PowerMode);
        fn set_polarity(polarity: Polarity);
        fn set_window_mode(en: bool);
        /// 锁定配置（LOCK 置位后 CSR 只读，直到复位）
        fn lock();
        fn is_locked() -> bool;
        /// 数字滤波
        fn set_filter(en: bool, count: u8);

        /// 写入迟滞（仅 COMP1 支持；COMP2 无此字段，默认空实现）
        #[inline]
        fn write_hyst(_: Hysteresis) {}
        /// 写入缩放器使能（仅 COMP1 支持；COMP2 无此字段，默认空实现）
        #[inline]
        fn write_scaler(_: bool) {}
    }

    macro_rules! impl_comp_instance {
        ($periph:ident, $pac:ident, $id:ident, $extras:ident) => {
            impl $crate::comp::hal::sealed::CompInstance for $crate::mcu::peripherals::$periph {
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
                    // 配置前先关闭比较器
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
                    $crate::comp::hal::sealed::impl_comp_instance!(@write_hyst $extras, $pac, hyst);
                }

                #[inline]
                fn write_scaler(en: bool) {
                    $crate::comp::hal::sealed::impl_comp_instance!(@write_scaler $extras, $pac, en);
                }
            }

            impl $crate::comp::Instance for $crate::mcu::peripherals::$periph {}
        };

        // COMP1 专有：迟滞与缩放器使能
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

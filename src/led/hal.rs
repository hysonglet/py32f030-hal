pub(super) mod sealed {
    use crate::pac;
    use crate::led::*;

    pub trait Instance {
        fn id() -> Id;

        #[inline]
        fn block() -> &'static pac::led::RegisterBlock {
            match Self::id() {
                Id::Led1 => unsafe { pac::LED::PTR.as_ref().unwrap() },
            }
        }

        #[inline]
        fn enable(en: bool) {
            Self::block().cr.modify(|_, w| w.ledon().bit(en));
        }

        #[inline]
        fn set_prescaler(pr: u8) {
            Self::block().pr.modify(|_, w| unsafe { w.pr().bits(pr) });
        }

        /// 选择 COM 数量（0=1 个 COM，1=2 个，2=3 个，3=4 个）
        #[inline]
        fn set_com_count(count: u8) {
            let sel = (count.saturating_sub(1)).min(3);
            Self::block()
                .cr
                .modify(|_, w| unsafe { w.led_com_sel().bits(sel) });
        }

        /// 设置显示时序 t1 / t2（各 8 位）
        #[inline]
        fn set_timing(t1: u8, t2: u8) {
            Self::block()
                .tr
                .modify(|_, w| unsafe { w.t1().bits(t1).t2().bits(t2) });
        }

        /// 亮度控制（EHS，2 位）
        #[inline]
        fn set_light_control(lc: LightControl) {
            Self::block()
                .cr
                .modify(|_, w| unsafe { w.ehs().bits(lc as u8) });
        }

        #[inline]
        fn set_interrupt(en: bool) {
            Self::block().cr.modify(|_, w| w.ie().bit(en));
        }

        #[inline]
        fn is_interrupt_pending() -> bool {
            Self::block().ir.read().flag().bit()
        }

        #[inline]
        fn clear_interrupt() {
            Self::block().ir.modify(|_, w| w.flag().bit(true));
        }

        /// 写入某个 COM 线的 8 段显示数据（dr0..dr3）
        #[inline]
        fn set_display_data(com: u8, seg: u32) {
            let block = Self::block();
            match com {
                0 => block.dr0.modify(|_, w| unsafe { w.bits(seg) }),
                1 => block.dr1.modify(|_, w| unsafe { w.bits(seg) }),
                2 => block.dr2.modify(|_, w| unsafe { w.bits(seg) }),
                3 => block.dr3.modify(|_, w| unsafe { w.bits(seg) }),
                _ => {}
            }
        }
    }
}

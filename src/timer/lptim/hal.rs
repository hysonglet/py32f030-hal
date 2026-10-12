use super::types::*;
use super::LpTimer;

pub(crate) mod sealed {
    use super::*;
    use crate::pac::lptim::RegisterBlock;

    pub trait Instance {
        /// 定时器索引
        fn id() -> LpTimer;

        #[inline]
        fn block() -> &'static RegisterBlock {
            unsafe { &*(crate::pac::LPTIM::ptr()) }
        }

        /// 选择内核时钟源（仅当 LPTIM 禁用时有效；写 RCC_CCIPR.LPTIMSEL）
        #[inline]
        fn set_clock_source(src: ClockSource) {
            let rcc = unsafe { crate::pac::RCC::PTR.as_ref().unwrap() };
            rcc.ccipr
                .modify(|_, w| unsafe { w.lptim1sel().bits(src as u8) });
        }

        /// 使能/禁止 LPTIM 内核
        #[inline]
        fn enable_kernel(en: bool) {
            Self::block().cr.modify(|_, w| w.enable().bit(en));
        }

        /// 启动单次计数（软件触发）：置 ENABLE 并写 SNGSTRT
        #[inline]
        fn start_single() {
            let b = Self::block();
            b.cr.modify(|_, w| w.enable().set_bit());
            b.cr.modify(|_, w| w.sngstrt().set_bit());
        }

        /// 启动连续计数（到 ARR 后自动重载并持续运行）
        #[inline]
        fn start_continuous() {
            Self::block()
                .cr
                .modify(|_, w| w.sngstrt().clear_bit().enable().set_bit());
        }

        /// 设置预分频系数（仅当 LPTIM 禁用时有效）
        #[inline]
        fn set_prescaler(p: Prescaler) {
            Self::block()
                .cfgr
                .modify(|_, w| unsafe { w.presc().bits(p as u8) });
        }

        /// 设置 ARR 预装载使能（仅当 LPTIM 禁用时有效）
        #[inline]
        fn set_preload(en: bool) {
            Self::block().cfgr.modify(|_, w| w.preload().bit(en));
        }

        /// 写自动重装载寄存器
        #[inline]
        fn set_auto_reload(arr: u16) {
            Self::block().arr.write(|w| unsafe { w.arr().bits(arr) });
        }

        /// 读自动重装载寄存器
        #[inline]
        fn get_auto_reload() -> u16 {
            Self::block().arr.read().arr().bits()
        }

        /// 读当前计数值
        #[inline]
        fn counter() -> u16 {
            Self::block().cnt.read().cnt().bits()
        }

        /// 停止计数（禁止内核）
        #[inline]
        fn stop() {
            Self::enable_kernel(false);
        }

        /// 使能/禁止中断事件
        #[inline]
        fn event_config(event: Event, en: bool) {
            match event {
                Event::AutoreloadMatch => {
                    Self::block().ier.modify(|_, w| w.arrmie().bit(en))
                }
            }
        }

        /// 查询事件标志
        #[inline]
        fn event_flag(event: Event) -> bool {
            match event {
                Event::AutoreloadMatch => Self::block().isr.read().arrm().bit_is_set(),
            }
        }

        /// 清除事件标志
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

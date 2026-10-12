//! 统一 APB 定时器实例层
//!
//! 设计要点：
//! 1. 核心寄存器操作写成 `sealed::Instance` 的**默认方法**，内部按 `TimerId` 分派到
//!    各自的寄存器块（TIM1/TIM3/TIM14/TIM16 的寄存器块类型不同，无法用统一的 `block()`）。
//!    每个实例只需实现 `fn id()`；`Self::id()` 在单态化后是常量，分派会被完全优化掉。
//! 2. 定时器**专有**能力（刹车、重复计数、编码器、中心对齐、单脉冲、TRGO、TI1 重映射）
//!    默认实现为空操作（no-op），仅由具备该硬件能力的实例覆写。
//!    这些能力由 [`crate::timer::capability`] 中的标记 trait 在**编译期**门控，
//!    因此用户不会调到 no-op，而是直接编译失败。

use super::types::*;
use crate::clock::peripheral::{PeripheralClockIndex, PeripheralIdToClockIndex};
use crate::clock::timer_pclk;
use crate::pac::{self, interrupt};

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        /// 定时器索引
        fn id() -> TimerId;

        /// TIM16 与 TIM17 功能完全一致且共用 `tim16` 寄存器块布局，
        /// 仅基址不同，故按索引返回对应的寄存器块引用。
        #[inline]
        fn tim16_block() -> &'static pac::tim16::RegisterBlock {
            match Self::id() {
                TimerId::TIM16 => unsafe { pac::TIM16::PTR.as_ref().unwrap() },
                TimerId::TIM17 => unsafe { pac::TIM17::PTR.as_ref().unwrap() },
                _ => unreachable!(),
            }
        }

        ////////////////////////// 核心（所有 APB 定时器共有） //////////////////////////

        /// 返回定时器外设时钟
        #[inline]
        fn get_time_pclk() -> u32 {
            timer_pclk()
        }

        /// 启动计数
        #[inline]
        fn start() {
            match Self::id() {
                TimerId::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() }
                    .cr1
                    .modify(|_, w| w.cen().set_bit()),
                TimerId::TIM3 => unsafe { pac::TIM3::PTR.as_ref().unwrap() }
                    .cr1
                    .modify(|_, w| w.cen().set_bit()),
                TimerId::TIM14 => unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                    .cr1
                    .modify(|_, w| w.cen().set_bit()),
                TimerId::TIM16 | TimerId::TIM17 => Self::tim16_block()
                    .cr1
                    .modify(|_, w| w.cen().set_bit()),
            }
        }

        /// 停止计数
        #[inline]
        fn stop() {
            match Self::id() {
                TimerId::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() }
                    .cr1
                    .modify(|_, w| w.cen().clear_bit()),
                TimerId::TIM3 => unsafe { pac::TIM3::PTR.as_ref().unwrap() }
                    .cr1
                    .modify(|_, w| w.cen().clear_bit()),
                TimerId::TIM14 => unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                    .cr1
                    .modify(|_, w| w.cen().clear_bit()),
                TimerId::TIM16 | TimerId::TIM17 => Self::tim16_block()
                    .cr1
                    .modify(|_, w| w.cen().clear_bit()),
            }
        }

        /// 设置预分频
        #[inline]
        fn set_prescaler(prescaler: u16) {
            match Self::id() {
                TimerId::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() }
                    .psc
                    .write(|w| unsafe { w.psc().bits(prescaler) }),
                TimerId::TIM3 => unsafe { pac::TIM3::PTR.as_ref().unwrap() }
                    .psc
                    .write(|w| unsafe { w.psc().bits(prescaler) }),
                TimerId::TIM14 => unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                    .psc
                    .write(|w| unsafe { w.psc().bits(prescaler) }),
                TimerId::TIM16 | TimerId::TIM17 => Self::tim16_block()
                    .psc
                    .write(|w| unsafe { w.psc().bits(prescaler) }),
            }
        }

        /// 设置周期计数值（ARR，保留其它位）
        #[inline]
        fn set_period_cnt(cnt: u16) {
            match Self::id() {
                TimerId::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() }
                    .arr
                    .modify(|_, w| unsafe { w.arr().bits(cnt) }),
                TimerId::TIM3 => unsafe { pac::TIM3::PTR.as_ref().unwrap() }
                    .arr
                    .modify(|_, w| unsafe { w.arr().bits(cnt) }),
                TimerId::TIM14 => unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                    .arr
                    .modify(|_, w| unsafe { w.arr().bits(cnt) }),
                TimerId::TIM16 | TimerId::TIM17 => Self::tim16_block()
                    .arr
                    .modify(|_, w| unsafe { w.arr().bits(cnt) }),
            }
        }

        /// 设置当前计数值
        #[inline]
        fn set_cnt(cnt: u16) {
            match Self::id() {
                TimerId::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() }
                    .cnt
                    .write(|w| unsafe { w.cnt().bits(cnt) }),
                TimerId::TIM3 => unsafe { pac::TIM3::PTR.as_ref().unwrap() }
                    .cnt
                    .write(|w| unsafe { w.cnt().bits(cnt) }),
                TimerId::TIM14 => unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                    .cnt
                    .write(|w| unsafe { w.cnt().bits(cnt) }),
                TimerId::TIM16 | TimerId::TIM17 => Self::tim16_block()
                    .cnt
                    .write(|w| unsafe { w.cnt().bits(cnt) }),
            }
        }

        /// 读取当前计数值
        #[inline]
        fn get_cnt() -> u16 {
            match Self::id() {
                TimerId::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() }
                    .cnt
                    .read()
                    .cnt()
                    .bits(),
                TimerId::TIM3 => unsafe { pac::TIM3::PTR.as_ref().unwrap() }
                    .cnt
                    .read()
                    .cnt()
                    .bits(),
                TimerId::TIM14 => unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                    .cnt
                    .read()
                    .cnt()
                    .bits(),
                TimerId::TIM16 | TimerId::TIM17 => Self::tim16_block()
                    .cnt
                    .read()
                    .cnt()
                    .bits(),
            }
        }

        /// 设置自动重装载值（ARR）
        #[inline]
        fn set_auto_reload(load: u16) {
            match Self::id() {
                TimerId::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() }
                    .arr
                    .write(|w| unsafe { w.arr().bits(load) }),
                TimerId::TIM3 => unsafe { pac::TIM3::PTR.as_ref().unwrap() }
                    .arr
                    .write(|w| unsafe { w.arr().bits(load) }),
                TimerId::TIM14 => unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                    .arr
                    .write(|w| unsafe { w.arr().bits(load) }),
                TimerId::TIM16 | TimerId::TIM17 => Self::tim16_block()
                    .arr
                    .write(|w| unsafe { w.arr().bits(load) }),
            }
        }

        /// 读取自动重装载值
        #[inline]
        fn get_reload() -> u16 {
            match Self::id() {
                TimerId::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() }
                    .arr
                    .read()
                    .arr()
                    .bits(),
                TimerId::TIM3 => unsafe { pac::TIM3::PTR.as_ref().unwrap() }
                    .arr
                    .read()
                    .arr()
                    .bits(),
                TimerId::TIM14 => unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                    .arr
                    .read()
                    .arr()
                    .bits(),
                TimerId::TIM16 | TimerId::TIM17 => Self::tim16_block()
                    .arr
                    .read()
                    .arr()
                    .bits(),
            }
        }

        /// 自动重装载预装载允许位（ARPE）
        #[inline]
        fn enable_auto_reload_buff(en: bool) {
            match Self::id() {
                TimerId::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() }
                    .cr1
                    .modify(|_, w| w.arpe().bit(en)),
                TimerId::TIM3 => unsafe { pac::TIM3::PTR.as_ref().unwrap() }
                    .cr1
                    .modify(|_, w| w.arpe().bit(en)),
                TimerId::TIM14 => unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                    .cr1
                    .modify(|_, w| w.arpe().bit(en)),
                TimerId::TIM16 | TimerId::TIM17 => Self::tim16_block()
                    .cr1
                    .modify(|_, w| w.arpe().bit(en)),
            }
        }

        /// 返回当前计数频率（CK_CNT）
        #[inline]
        fn counter_frequency() -> u32 {
            let psc: u32 = match Self::id() {
                TimerId::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() }.psc.read().bits(),
                TimerId::TIM3 => unsafe { pac::TIM3::PTR.as_ref().unwrap() }.psc.read().bits(),
                TimerId::TIM14 => unsafe { pac::TIM14::PTR.as_ref().unwrap() }.psc.read().bits(),
                TimerId::TIM16 | TimerId::TIM17 => Self::tim16_block().psc.read().bits(),
            };
            // CK_CNT = fCK_PSC / (PSC[15:0] + 1)
            timer_pclk() / (psc + 1)
        }

        /// 设置时钟分频因子 tDTS（CR1.CKD）
        #[inline]
        fn set_clock_division(div: ClockDiv) {
            match Self::id() {
                TimerId::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() }
                    .cr1
                    .modify(|_, w| unsafe { w.ckd().bits(div as u8) }),
                TimerId::TIM3 => unsafe { pac::TIM3::PTR.as_ref().unwrap() }
                    .cr1
                    .modify(|_, w| unsafe { w.ckd().bits(div as u8) }),
                TimerId::TIM14 => unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                    .cr1
                    .modify(|_, w| unsafe { w.ckd().bits(div as u8) }),
                TimerId::TIM16 | TimerId::TIM17 => Self::tim16_block()
                    .cr1
                    .modify(|_, w| unsafe { w.ckd().bits(div as u8) }),
            }
        }

        ////////////////////////// 通道相关 //////////////////////////

        /// 设置通道输入/输出类型（CCxS）
        fn set_channel_type(channel: Channel, channel_type: ChannelType) {
            let v = channel_type as u8;
            match Self::id() {
                TimerId::TIM1 => {
                    let b = unsafe { pac::TIM1::PTR.as_ref().unwrap() };
                    match channel {
                        Channel::CH1 => {
                            b.ccmr1_output().modify(|_, w| unsafe { w.cc1s().bits(v) })
                        }
                        Channel::CH2 => {
                            b.ccmr1_output().modify(|_, w| unsafe { w.cc2s().bits(v) })
                        }
                        Channel::CH3 => {
                            b.ccmr2_output().modify(|_, w| unsafe { w.cc3s().bits(v) })
                        }
                        Channel::CH4 => {
                            b.ccmr2_output().modify(|_, w| unsafe { w.cc4s().bits(v) })
                        }
                    }
                }
                TimerId::TIM3 => {
                    let b = unsafe { pac::TIM3::PTR.as_ref().unwrap() };
                    match channel {
                        Channel::CH1 => {
                            b.ccmr1_output().modify(|_, w| unsafe { w.cc1s().bits(v) })
                        }
                        Channel::CH2 => {
                            b.ccmr1_output().modify(|_, w| unsafe { w.cc2s().bits(v) })
                        }
                        Channel::CH3 => {
                            b.ccmr2_output().modify(|_, w| unsafe { w.cc3s().bits(v) })
                        }
                        Channel::CH4 => {
                            b.ccmr2_output().modify(|_, w| unsafe { w.cc4s().bits(v) })
                        }
                    }
                }
                // 单通道定时器：仅 CH1 有效
                TimerId::TIM14 => {
                    if matches!(channel, Channel::CH1) {
                        unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                            .ccmr1_output()
                            .modify(|_, w| unsafe { w.cc1s().bits(v) });
                    }
                }
                TimerId::TIM16 | TimerId::TIM17 => {
                    if matches!(channel, Channel::CH1) {
                        Self::tim16_block()
                            .ccmr1_output()
                            .modify(|_, w| unsafe { w.cc1s().bits(v) });
                    }
                }
            }
        }

        /// 设置通道输出比较模式（OCxM / OCxCE / OCxFE / OCxPE）
        #[allow(clippy::too_many_arguments)]
        fn set_channel_output_config(
            channel: Channel,
            mode: ChannelMode,
            clear: bool,
            fast: bool,
            preload: bool,
        ) {
            let m = mode as u8;
            match Self::id() {
                TimerId::TIM1 => {
                    let b = unsafe { pac::TIM1::PTR.as_ref().unwrap() };
                    match channel {
                        Channel::CH1 => b.ccmr1_output().modify(|_, w| unsafe {
                            w.oc1m().bits(m).oc1ce().bit(clear).oc1fe().bit(fast).oc1pe().bit(preload)
                        }),
                        Channel::CH2 => b.ccmr1_output().modify(|_, w| unsafe {
                            w.oc2m().bits(m).oc2ce().bit(clear).oc2fe().bit(fast).oc2pe().bit(preload)
                        }),
                        Channel::CH3 => b.ccmr2_output().modify(|_, w| unsafe {
                            w.oc3m().bits(m).oc3ce().bit(clear).oc3fe().bit(fast).oc3pe().bit(preload)
                        }),
                        Channel::CH4 => b.ccmr2_output().modify(|_, w| unsafe {
                            w.oc4m().bits(m).oc4ce().bit(clear).oc4fe().bit(fast).oc4pe().bit(preload)
                        }),
                    }
                }
                TimerId::TIM3 => {
                    let b = unsafe { pac::TIM3::PTR.as_ref().unwrap() };
                    match channel {
                        Channel::CH1 => b.ccmr1_output().modify(|_, w| unsafe {
                            w.oc1m().bits(m).oc1ce().bit(clear).oc1fe().bit(fast).oc1pe().bit(preload)
                        }),
                        Channel::CH2 => b.ccmr1_output().modify(|_, w| unsafe {
                            w.oc2m().bits(m).oc2ce().bit(clear).oc2fe().bit(fast).oc2pe().bit(preload)
                        }),
                        Channel::CH3 => b.ccmr2_output().modify(|_, w| unsafe {
                            w.oc3m().bits(m).oc3ce().bit(clear).oc3fe().bit(fast).oc3pe().bit(preload)
                        }),
                        Channel::CH4 => b.ccmr2_output().modify(|_, w| unsafe {
                            w.oc4m().bits(m).oc4ce().bit(clear).oc4fe().bit(fast).oc4pe().bit(preload)
                        }),
                    }
                }
                // TIM14 无 OC1CE 位
                TimerId::TIM14 => {
                    if matches!(channel, Channel::CH1) {
                        unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                            .ccmr1_output()
                            .modify(|_, w| unsafe {
                                w.oc1m().bits(m).oc1fe().bit(fast).oc1pe().bit(preload)
                            });
                    }
                }
                // TIM16 同样无 OC1CE 位
                TimerId::TIM16 | TimerId::TIM17 => {
                    if matches!(channel, Channel::CH1) {
                        Self::tim16_block()
                            .ccmr1_output()
                            .modify(|_, w| unsafe {
                                w.oc1m().bits(m).oc1fe().bit(fast).oc1pe().bit(preload)
                            });
                    }
                }
            }
        }

        /// 设置通道捕获/比较值（CCRx）
        fn set_channel_compare(channel: Channel, ccr: u16) {
            match Self::id() {
                TimerId::TIM1 => {
                    let b = unsafe { pac::TIM1::PTR.as_ref().unwrap() };
                    match channel {
                        Channel::CH1 => b.ccr1.write(|w| unsafe { w.ccr1().bits(ccr) }),
                        Channel::CH2 => b.ccr2.write(|w| unsafe { w.ccr2().bits(ccr) }),
                        Channel::CH3 => b.ccr3.write(|w| unsafe { w.ccr3().bits(ccr) }),
                        Channel::CH4 => b.ccr4.write(|w| unsafe { w.ccr4().bits(ccr) }),
                    }
                }
                TimerId::TIM3 => {
                    let b = unsafe { pac::TIM3::PTR.as_ref().unwrap() };
                    match channel {
                        Channel::CH1 => b.ccr1.write(|w| unsafe { w.ccr1().bits(ccr) }),
                        Channel::CH2 => b.ccr2.write(|w| unsafe { w.ccr2().bits(ccr) }),
                        Channel::CH3 => b.ccr3.write(|w| unsafe { w.ccr3().bits(ccr) }),
                        Channel::CH4 => b.ccr4.write(|w| unsafe { w.ccr4().bits(ccr) }),
                    }
                }
                TimerId::TIM14 => {
                    if matches!(channel, Channel::CH1) {
                        unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                            .ccr1
                            .write(|w| unsafe { w.ccr1().bits(ccr) });
                    }
                }
                TimerId::TIM16 | TimerId::TIM17 => {
                    if matches!(channel, Channel::CH1) {
                        Self::tim16_block()
                            .ccr1
                            .write(|w| unsafe { w.ccr1().bits(ccr) });
                    }
                }
            }
        }

        /// 读取通道捕获值（CCRx）
        fn get_channel_capture(channel: Channel) -> u16 {
            match Self::id() {
                TimerId::TIM1 => {
                    let b = unsafe { pac::TIM1::PTR.as_ref().unwrap() };
                    match channel {
                        Channel::CH1 => b.ccr1.read().ccr1().bits(),
                        Channel::CH2 => b.ccr2.read().ccr2().bits(),
                        Channel::CH3 => b.ccr3.read().ccr3().bits(),
                        Channel::CH4 => b.ccr4.read().ccr4().bits(),
                    }
                }
                TimerId::TIM3 => {
                    let b = unsafe { pac::TIM3::PTR.as_ref().unwrap() };
                    match channel {
                        Channel::CH1 => b.ccr1.read().ccr1().bits(),
                        Channel::CH2 => b.ccr2.read().ccr2().bits(),
                        Channel::CH3 => b.ccr3.read().ccr3().bits(),
                        Channel::CH4 => b.ccr4.read().ccr4().bits(),
                    }
                }
                TimerId::TIM14 => unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                    .ccr1
                    .read()
                    .ccr1()
                    .bits(),
                TimerId::TIM16 | TimerId::TIM17 => Self::tim16_block()
                    .ccr1
                    .read()
                    .ccr1()
                    .bits(),
            }
        }

        /// 设置输出通道有效极性 / 输出使能 / 空闲态（OIS）
        fn set_channel_output_effective_level(
            channel: Channel,
            channel_output: ChannelOutput,
            polarity: bool,
            state: bool,
            idle: bool,
        ) {
            match Self::id() {
                TimerId::TIM1 => {
                    let b = unsafe { pac::TIM1::PTR.as_ref().unwrap() };
                    match (channel, channel_output) {
                        (Channel::CH1, ChannelOutput::P) => {
                            b.ccer.modify(|_, w| w.cc1p().bit(!polarity).cc1e().bit(state));
                            b.cr2.modify(|_, w| w.ois1().bit(idle));
                        }
                        (Channel::CH2, ChannelOutput::P) => {
                            b.ccer.modify(|_, w| w.cc2p().bit(!polarity).cc2e().bit(state));
                            b.cr2.modify(|_, w| w.ois2().bit(idle));
                        }
                        (Channel::CH3, ChannelOutput::P) => {
                            b.ccer.modify(|_, w| w.cc3p().bit(!polarity).cc3e().bit(state));
                            b.cr2.modify(|_, w| w.ois3().bit(idle));
                        }
                        (Channel::CH4, ChannelOutput::P) => {
                            b.ccer.modify(|_, w| w.cc4p().bit(!polarity).cc4e().bit(state));
                            b.cr2.modify(|_, w| w.ois4().bit(idle));
                        }
                        (Channel::CH1, ChannelOutput::N) => {
                            b.ccer.modify(|_, w| w.cc1np().bit(!polarity).cc1ne().bit(state));
                            b.cr2.modify(|_, w| w.ois1n().bit(idle));
                        }
                        (Channel::CH2, ChannelOutput::N) => {
                            b.ccer.modify(|_, w| w.cc2np().bit(!polarity).cc2ne().bit(state));
                            b.cr2.modify(|_, w| w.ois2n().bit(idle));
                        }
                        (Channel::CH3, ChannelOutput::N) => {
                            b.ccer.modify(|_, w| w.cc3np().bit(!polarity).cc3ne().bit(state));
                            b.cr2.modify(|_, w| w.ois3n().bit(idle));
                        }
                        // TIM1 无 CH4N
                        (Channel::CH4, ChannelOutput::N) => {}
                    }
                }
                // TIM3 无互补输出、无 OIS
                TimerId::TIM3 => {
                    let b = unsafe { pac::TIM3::PTR.as_ref().unwrap() };
                    match channel {
                        Channel::CH1 => b.ccer.modify(|_, w| w.cc1p().bit(!polarity).cc1e().bit(state)),
                        Channel::CH2 => b.ccer.modify(|_, w| w.cc2p().bit(!polarity).cc2e().bit(state)),
                        Channel::CH3 => b.ccer.modify(|_, w| w.cc3p().bit(!polarity).cc3e().bit(state)),
                        Channel::CH4 => b.ccer.modify(|_, w| w.cc4p().bit(!polarity).cc4e().bit(state)),
                    }
                }
                // TIM14 无 CR2（无 OIS），仅 CH1P
                TimerId::TIM14 => {
                    if matches!((channel, channel_output), (Channel::CH1, ChannelOutput::P)) {
                        unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                            .ccer
                            .modify(|_, w| w.cc1p().bit(!polarity).cc1e().bit(state));
                    }
                }
                // TIM16 仅 CH1，支持互补与 OIS1/OIS1N
                TimerId::TIM16 | TimerId::TIM17 => {
                    let b = Self::tim16_block();
                    match (channel, channel_output) {
                        (Channel::CH1, ChannelOutput::P) => {
                            b.ccer.modify(|_, w| w.cc1p().bit(!polarity).cc1e().bit(state));
                            b.cr2.modify(|_, w| w.ois1().bit(idle));
                        }
                        (Channel::CH1, ChannelOutput::N) => {
                            b.ccer.modify(|_, w| w.cc1np().bit(!polarity).cc1ne().bit(state));
                            b.cr2.modify(|_, w| w.ois1n().bit(idle));
                        }
                        _ => {}
                    }
                }
            }
        }

        /// 单独使能/关闭某通道输出（CCxE / CCxNE）
        fn set_enable_channel(channel: Channel, channel_output: ChannelOutput, en: bool) {
            match Self::id() {
                TimerId::TIM1 => {
                    let b = unsafe { pac::TIM1::PTR.as_ref().unwrap() };
                    match (channel, channel_output) {
                        (Channel::CH1, ChannelOutput::P) => b.ccer.modify(|_, w| w.cc1e().bit(en)),
                        (Channel::CH2, ChannelOutput::P) => b.ccer.modify(|_, w| w.cc2e().bit(en)),
                        (Channel::CH3, ChannelOutput::P) => b.ccer.modify(|_, w| w.cc3e().bit(en)),
                        (Channel::CH4, ChannelOutput::P) => b.ccer.modify(|_, w| w.cc4e().bit(en)),
                        (Channel::CH1, ChannelOutput::N) => b.ccer.modify(|_, w| w.cc1ne().bit(en)),
                        (Channel::CH2, ChannelOutput::N) => b.ccer.modify(|_, w| w.cc2ne().bit(en)),
                        (Channel::CH3, ChannelOutput::N) => b.ccer.modify(|_, w| w.cc3ne().bit(en)),
                        (Channel::CH4, ChannelOutput::N) => {}
                    }
                }
                TimerId::TIM3 => {
                    let b = unsafe { pac::TIM3::PTR.as_ref().unwrap() };
                    match channel {
                        Channel::CH1 => b.ccer.modify(|_, w| w.cc1e().bit(en)),
                        Channel::CH2 => b.ccer.modify(|_, w| w.cc2e().bit(en)),
                        Channel::CH3 => b.ccer.modify(|_, w| w.cc3e().bit(en)),
                        Channel::CH4 => b.ccer.modify(|_, w| w.cc4e().bit(en)),
                    }
                }
                // 单通道且无互补
                TimerId::TIM14 => {
                    if matches!((channel, channel_output), (Channel::CH1, ChannelOutput::P)) {
                        unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                            .ccer
                            .modify(|_, w| w.cc1e().bit(en));
                    }
                }
                TimerId::TIM16 | TimerId::TIM17 => {
                    let b = Self::tim16_block();
                    match (channel, channel_output) {
                        (Channel::CH1, ChannelOutput::P) => b.ccer.modify(|_, w| w.cc1e().bit(en)),
                        (Channel::CH1, ChannelOutput::N) => b.ccer.modify(|_, w| w.cc1ne().bit(en)),
                        _ => {}
                    }
                }
            }
        }

        /// 配置通道为输入捕获
        fn set_channel_input_config(
            channel: Channel,
            selection: InputCaptureSelection,
            polarity: InputCapturePolarity,
            prescaler: InputCapturePrescaler,
            filter: InputCaptureFilter,
        ) {
            let (p, np) = Self::input_polarity_bits(polarity);
            let sel = selection as u8;
            let psc = prescaler as u8;
            let f = filter.value();
            match Self::id() {
                TimerId::TIM1 => {
                    let b = unsafe { pac::TIM1::PTR.as_ref().unwrap() };
                    match channel {
                        // 注意：TIM1 的 CH1 输入预分频字段名为 icpsc（无数字）
                        Channel::CH1 => {
                            b.ccmr1_input().modify(|_, w| unsafe {
                                w.cc1s().bits(sel).icpsc().bits(psc).ic1f().bits(f)
                            });
                            b.ccer.modify(|_, w| w.cc1p().bit(p).cc1np().bit(np));
                        }
                        Channel::CH2 => {
                            b.ccmr1_input().modify(|_, w| unsafe {
                                w.cc2s().bits(sel).ic2psc().bits(psc).ic2f().bits(f)
                            });
                            b.ccer.modify(|_, w| w.cc2p().bit(p).cc2np().bit(np));
                        }
                        Channel::CH3 => {
                            b.ccmr2_input().modify(|_, w| unsafe {
                                w.cc3s().bits(sel).ic3psc().bits(psc).ic3f().bits(f)
                            });
                            b.ccer.modify(|_, w| w.cc3p().bit(p).cc3np().bit(np));
                        }
                        Channel::CH4 => {
                            b.ccmr2_input().modify(|_, w| unsafe {
                                w.cc4s().bits(sel).ic4psc().bits(psc).ic4f().bits(f)
                            });
                            // CH4 无互补输入
                            b.ccer.modify(|_, w| w.cc4p().bit(p));
                        }
                    }
                }
                TimerId::TIM3 => {
                    let b = unsafe { pac::TIM3::PTR.as_ref().unwrap() };
                    match channel {
                        Channel::CH1 => {
                            b.ccmr1_input().modify(|_, w| unsafe {
                                w.cc1s().bits(sel).ic1psc().bits(psc).ic1f().bits(f)
                            });
                            b.ccer.modify(|_, w| w.cc1p().bit(p));
                        }
                        Channel::CH2 => {
                            b.ccmr1_input().modify(|_, w| unsafe {
                                w.cc2s().bits(sel).ic2psc().bits(psc).ic2f().bits(f)
                            });
                            b.ccer.modify(|_, w| w.cc2p().bit(p));
                        }
                        Channel::CH3 => {
                            b.ccmr2_input().modify(|_, w| unsafe {
                                w.cc3s().bits(sel).ic3psc().bits(psc).ic3f().bits(f)
                            });
                            b.ccer.modify(|_, w| w.cc3p().bit(p));
                        }
                        Channel::CH4 => {
                            b.ccmr2_input().modify(|_, w| unsafe {
                                w.cc4s().bits(sel).ic4psc().bits(psc).ic4f().bits(f)
                            });
                            b.ccer.modify(|_, w| w.cc4p().bit(p));
                        }
                    }
                }
                // 单通道，仅 cc1p（TIM14 无 CC1NP）
                TimerId::TIM14 => {
                    if matches!(channel, Channel::CH1) {
                        let b = unsafe { pac::TIM14::PTR.as_ref().unwrap() };
                        b.ccmr1_input()
                            .modify(|_, w| unsafe { w.cc1s().bits(sel).ic1psc().bits(psc).ic1f().bits(f) });
                        b.ccer.modify(|_, w| w.cc1p().bit(p));
                    }
                }
                TimerId::TIM16 | TimerId::TIM17 => {
                    if matches!(channel, Channel::CH1) {
                        let b = Self::tim16_block();
                        b.ccmr1_input().modify(|_, w| unsafe {
                            w.cc1s().bits(sel).ic1psc().bits(psc).ic1f().bits(f)
                        });
                        b.ccer.modify(|_, w| w.cc1p().bit(p).cc1np().bit(np));
                    }
                }
            }
        }

        /// 输入捕获极性 → CCER 的 CCxP / CCxNP
        #[inline]
        fn input_polarity_bits(polarity: InputCapturePolarity) -> (bool, bool) {
            match polarity {
                InputCapturePolarity::Rising => (false, false),
                InputCapturePolarity::Falling => (true, false),
                InputCapturePolarity::BothEdge => (true, true),
            }
        }

        ////////////////////////// 事件与软件触发 //////////////////////////

        /// 软件产生事件（EGR）
        fn triggle(signal: Triggle) {
            match Self::id() {
                TimerId::TIM1 => {
                    let egr = &unsafe { pac::TIM1::PTR.as_ref().unwrap() }.egr;
                    match signal {
                        Triggle::UG => egr.write(|w| w.ug().set_bit()),
                        Triggle::CC1G => egr.write(|w| w.cc1g().set_bit()),
                        Triggle::CC2G => egr.write(|w| w.cc2g().set_bit()),
                        Triggle::CC3G => egr.write(|w| w.cc3g().set_bit()),
                        Triggle::CC4G => egr.write(|w| w.cc4g().set_bit()),
                        Triggle::COMG => egr.write(|w| w.comg().set_bit()),
                        Triggle::TG => egr.write(|w| w.tg().set_bit()),
                        Triggle::BG => egr.write(|w| w.bg().set_bit()),
                    }
                }
                // TIM3 无 COMG / BG
                TimerId::TIM3 => {
                    let egr = &unsafe { pac::TIM3::PTR.as_ref().unwrap() }.egr;
                    match signal {
                        Triggle::UG => egr.write(|w| w.ug().set_bit()),
                        Triggle::CC1G => egr.write(|w| w.cc1g().set_bit()),
                        Triggle::CC2G => egr.write(|w| w.cc2g().set_bit()),
                        Triggle::CC3G => egr.write(|w| w.cc3g().set_bit()),
                        Triggle::CC4G => egr.write(|w| w.cc4g().set_bit()),
                        Triggle::TG => egr.write(|w| w.tg().set_bit()),
                        _ => {}
                    }
                }
                // TIM14 仅 UG / CC1G
                TimerId::TIM14 => {
                    let egr = &unsafe { pac::TIM14::PTR.as_ref().unwrap() }.egr;
                    match signal {
                        Triggle::UG => egr.write(|w| w.ug().set_bit()),
                        Triggle::CC1G => egr.write(|w| w.cc1g().set_bit()),
                        _ => {}
                    }
                }
                // TIM16 支持 UG / CC1G / COMG / BG
                TimerId::TIM16 | TimerId::TIM17 => {
                    let egr = &Self::tim16_block().egr;
                    match signal {
                        Triggle::UG => egr.write(|w| w.ug().set_bit()),
                        Triggle::CC1G => egr.write(|w| w.cc1g().set_bit()),
                        Triggle::COMG => egr.write(|w| w.comg().set_bit()),
                        Triggle::BG => egr.write(|w| w.bg().set_bit()),
                        _ => {}
                    }
                }
            }
        }

        /// 事件标志
        fn event_flag(event: Event) -> bool {
            match Self::id() {
                TimerId::TIM1 => {
                    let sr = unsafe { pac::TIM1::PTR.as_ref().unwrap() }.sr.read();
                    match event {
                        Event::UIF => sr.uif().bit(),
                        Event::CC1IF => sr.cc1if().bit(),
                        Event::CC2IF => sr.cc2if().bit(),
                        Event::CC3IF => sr.cc3if().bit(),
                        Event::CC4IF => sr.cc4if().bit(),
                        Event::COMIF => sr.comif().bit(),
                        Event::TIF => sr.tif().bit(),
                        Event::BIF => sr.bif().bit(),
                        Event::CC1OF => sr.cc1of().bit(),
                        Event::CC2OF => sr.cc2of().bit(),
                        Event::CC3OF => sr.cc3of().bit(),
                        Event::CC4OF => sr.cc4of().bit(),
                    }
                }
                // TIM3 无 COMIF / BIF
                TimerId::TIM3 => {
                    let sr = unsafe { pac::TIM3::PTR.as_ref().unwrap() }.sr.read();
                    match event {
                        Event::UIF => sr.uif().bit(),
                        Event::CC1IF => sr.cc1if().bit(),
                        Event::CC2IF => sr.cc2if().bit(),
                        Event::CC3IF => sr.cc3if().bit(),
                        Event::CC4IF => sr.cc4if().bit(),
                        Event::TIF => sr.tif().bit(),
                        Event::CC1OF => sr.cc1of().bit(),
                        Event::CC2OF => sr.cc2of().bit(),
                        Event::CC3OF => sr.cc3of().bit(),
                        Event::CC4OF => sr.cc4of().bit(),
                        _ => false,
                    }
                }
                // TIM14 仅 UIF / CC1IF / CC1OF
                TimerId::TIM14 => {
                    let sr = unsafe { pac::TIM14::PTR.as_ref().unwrap() }.sr.read();
                    match event {
                        Event::UIF => sr.uif().bit(),
                        Event::CC1IF => sr.cc1if().bit(),
                        Event::CC1OF => sr.cc1of().bit(),
                        _ => false,
                    }
                }
                // TIM16：UIF / CC1IF / COMIF / BIF / CC1OF
                TimerId::TIM16 | TimerId::TIM17 => {
                    let sr = Self::tim16_block().sr.read();
                    match event {
                        Event::UIF => sr.uif().bit(),
                        Event::CC1IF => sr.cc1if().bit(),
                        Event::COMIF => sr.comif().bit(),
                        Event::BIF => sr.bif().bit(),
                        Event::CC1OF => sr.cc1of().bit(),
                        _ => false,
                    }
                }
            }
        }

        /// 事件中断使能/屏蔽（DIER）
        fn event_config(event: Event, en: bool) {
            match Self::id() {
                TimerId::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() }
                    .dier
                    .modify(|_, w| match event {
                        Event::UIF => w.uie().bit(en),
                        Event::CC1IF => w.cc1ie().bit(en),
                        Event::CC2IF => w.cc2ie().bit(en),
                        Event::CC3IF => w.cc3ie().bit(en),
                        Event::CC4IF => w.cc4ie().bit(en),
                        Event::COMIF => w.comie().bit(en),
                        Event::TIF => w.tie().bit(en),
                        Event::BIF => w.bie().bit(en),
                        Event::CC1OF => w.cc1de().bit(en),
                        Event::CC2OF => w.cc2de().bit(en),
                        Event::CC3OF => w.cc3de().bit(en),
                        Event::CC4OF => w.cc4de().bit(en),
                    }),
                TimerId::TIM3 => unsafe { pac::TIM3::PTR.as_ref().unwrap() }.dier.modify(
                    |_, w| match event {
                        Event::UIF => w.uie().bit(en),
                        Event::CC1IF => w.cc1ie().bit(en),
                        Event::CC2IF => w.cc2ie().bit(en),
                        Event::CC3IF => w.cc3ie().bit(en),
                        Event::CC4IF => w.cc4ie().bit(en),
                        Event::TIF => w.tie().bit(en),
                        Event::CC1OF => w.cc1de().bit(en),
                        Event::CC2OF => w.cc2de().bit(en),
                        Event::CC3OF => w.cc3de().bit(en),
                        Event::CC4OF => w.cc4de().bit(en),
                        _ => w,
                    },
                ),
                // TIM14 DIER 无 CC1DE，过捕获随 CC1IE 一起使能
                TimerId::TIM14 => unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                    .dier
                    .modify(|_, w| match event {
                        Event::UIF => w.uie().bit(en),
                        Event::CC1IF | Event::CC1OF => w.cc1ie().bit(en),
                        _ => w,
                    }),
                TimerId::TIM16 | TimerId::TIM17 => Self::tim16_block()
                    .dier
                    .modify(|_, w| match event {
                        Event::UIF => w.uie().bit(en),
                        Event::CC1IF | Event::CC1OF => w.cc1ie().bit(en),
                        Event::COMIF => w.comie().bit(en),
                        Event::BIF => w.bie().bit(en),
                        _ => w,
                    }),
            }
        }

        /// 事件标志清除（SR）
        fn event_clear(event: Event) {
            match Self::id() {
                TimerId::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() }
                    .sr
                    .modify(|_, w| match event {
                        Event::UIF => w.uif().clear_bit(),
                        Event::CC1IF => w.cc1if().clear_bit(),
                        Event::CC2IF => w.cc2if().clear_bit(),
                        Event::CC3IF => w.cc3if().clear_bit(),
                        Event::CC4IF => w.cc4if().clear_bit(),
                        Event::COMIF => w.comif().clear_bit(),
                        Event::TIF => w.tif().clear_bit(),
                        Event::BIF => w.bif().clear_bit(),
                        Event::CC1OF => w.cc1of().clear_bit(),
                        Event::CC2OF => w.cc2of().clear_bit(),
                        Event::CC3OF => w.cc3of().clear_bit(),
                        Event::CC4OF => w.cc4of().clear_bit(),
                    }),
                TimerId::TIM3 => unsafe { pac::TIM3::PTR.as_ref().unwrap() }
                    .sr
                    .modify(|_, w| match event {
                        Event::UIF => w.uif().clear_bit(),
                        Event::CC1IF => w.cc1if().clear_bit(),
                        Event::CC2IF => w.cc2if().clear_bit(),
                        Event::CC3IF => w.cc3if().clear_bit(),
                        Event::CC4IF => w.cc4if().clear_bit(),
                        Event::TIF => w.tif().clear_bit(),
                        Event::CC1OF => w.cc1of().clear_bit(),
                        Event::CC2OF => w.cc2of().clear_bit(),
                        Event::CC3OF => w.cc3of().clear_bit(),
                        Event::CC4OF => w.cc4of().clear_bit(),
                        _ => w,
                    }),
                TimerId::TIM14 => unsafe { pac::TIM14::PTR.as_ref().unwrap() }
                    .sr
                    .modify(|_, w| match event {
                        Event::UIF => w.uif().clear_bit(),
                        Event::CC1IF => w.cc1if().clear_bit(),
                        Event::CC1OF => w.cc1of().clear_bit(),
                        _ => w,
                    }),
                TimerId::TIM16 | TimerId::TIM17 => Self::tim16_block()
                    .sr
                    .modify(|_, w| match event {
                        Event::UIF => w.uif().clear_bit(),
                        Event::CC1IF => w.cc1if().clear_bit(),
                        Event::COMIF => w.comif().clear_bit(),
                        Event::BIF => w.bif().clear_bit(),
                        Event::CC1OF => w.cc1of().clear_bit(),
                        _ => w,
                    }),
            }
        }

        ////////////////////////// 时长换算 //////////////////////////

        /// 由微秒计算 (PSC, RCR, ARR)
        #[inline]
        fn micros_to_compute_with_rep(micros: u64) -> (u16, u8, u16) {
            let ticks = micros * Self::get_time_pclk() as u64 / 1_000_000;
            let psc = ticks / (1u64 << 24);
            let count = ticks / (psc + 1);
            let rep = count / (1u64 << 16);
            let arr = count / (rep + 1);
            (psc as u16, rep as u8, arr as u16)
        }

        /// 由纳秒计算 (PSC, RCR, ARR)
        #[inline]
        fn nanosecond_to_compute_with_rep(nano: u64) -> (u16, u8, u16) {
            let ticks = nano * Self::get_time_pclk() as u64 / 1_000_000_000;
            let psc = ticks / (1u64 << 24);
            let count = ticks / (psc + 1);
            let rep = count / (1u64 << 16);
            let arr = count / (rep + 1);
            (psc as u16, rep as u8, arr as u16)
        }

        ////////////////////////// 以下为"能力"方法 //////////////////////////
        //// 默认均为空操作（no-op）。仅有对应硬件的实例会覆写它们，
        //// 且高层 API 通过 capability trait 在编译期门控，用户不会调到 no-op。

        /// 使能主输出（BDTR.MOE），仅 TIM1 / TIM16
        #[inline]
        fn enable_channel_output(_en: bool) {}

        /// 设置中心对齐模式，仅 TIM1 / TIM3
        #[inline]
        fn set_cms(_cms: CenterAlignedMode) {}

        /// 设置计数方向，仅 TIM1 / TIM3
        #[inline]
        fn set_dir(_dir: CountDirection) {}

        /// 单脉冲模式（OPM），TIM1 / TIM3 / TIM16
        #[inline]
        fn enable_single_mode(_en: bool) {}

        /// 重复计数（RCR），仅 TIM1 / TIM16
        #[inline]
        fn set_repetition(_rep: u8) {}

        /// 刹车与死区（BDTR），仅 TIM1 / TIM16
        #[allow(clippy::too_many_arguments)]
        fn config_break_deadtime(
            _dead_time: DeadTime,
            _lock_level: LockLevel,
            _off_state_run: OffState,
            _off_state_idle: OffState,
            _break_polarity: BreakPolarity,
            _break_enable: bool,
            _automatic_output: bool,
        ) {
        }

        /// 编码器模式（SMCR），仅 TIM1 / TIM3
        #[inline]
        fn set_encoder_mode(_mode: EncoderMode) {}

        /// 从模式（SMCR.SMS），仅 TIM1 / TIM3
        #[inline]
        fn set_slave_mode(_mode: SlaveMode) {}

        /// 触发输入选择（SMCR.TS），仅 TIM1 / TIM3
        #[inline]
        fn set_trigger_selection(_ts: TriggerSelection) {}

        /// 主/从模式同步（SMCR.MSM），仅 TIM1 / TIM3
        #[inline]
        fn set_master_slave_mode(_en: bool) {}

        /// OCREF 清除源（SMCR.OCCS），仅 TIM1 / TIM3
        #[inline]
        fn set_ocref_clear_source(_source: OcRefClearSource) {}

        /// 主模式触发输出 TRGO（CR2.MMS），仅 TIM1 / TIM3
        #[inline]
        fn set_master_output_trigger(_trigger: MasterOutputTrigger) {}

        /// TI1 输入重映射（OR），仅 TIM14
        #[inline]
        fn set_ti1_remap(_remap: Ti1Remap) {}
    }
}

//////////////////////// 时钟索引与中断 ////////////////////////

impl PeripheralIdToClockIndex for TimerId {
    fn clock(&self) -> PeripheralClockIndex {
        match *self {
            TimerId::TIM1 => PeripheralClockIndex::TIM1,
            TimerId::TIM3 => PeripheralClockIndex::TIM3,
            TimerId::TIM14 => PeripheralClockIndex::TIM14,
            TimerId::TIM16 => PeripheralClockIndex::TIM16,
            TimerId::TIM17 => PeripheralClockIndex::TIM17,
        }
    }
}

impl TimerId {
    /// 该定时器占用的全部 NVIC 中断向量
    ///
    /// TIM1 有**两个**中断向量（BRK_UP_TRG_COM 与 CC），其余各一个。
    pub fn interrupts(&self) -> &'static [interrupt] {
        match *self {
            TimerId::TIM1 => &[interrupt::TIM1_BRK_UP_TRG_COM, interrupt::TIM1_CC],
            TimerId::TIM3 => &[interrupt::TIM3],
            TimerId::TIM14 => &[interrupt::TIM14],
            TimerId::TIM16 => &[interrupt::TIM16],
            TimerId::TIM17 => &[interrupt::TIM17],
        }
    }

    /// 使能该定时器全部中断向量（NVIC unmask）
    #[inline]
    pub fn enable_interrupts(&self) {
        for irq in self.interrupts() {
            unsafe { cortex_m::peripheral::NVIC::unmask(*irq) }
        }
    }

    /// 关闭该定时器全部中断向量（NVIC mask）
    #[inline]
    pub fn disable_interrupts(&self) {
        for irq in self.interrupts() {
            cortex_m::peripheral::NVIC::mask(*irq)
        }
    }
}

//////////////////////// TIM1：全功能（核心 id + 全部能力覆写） ////////////////////////

impl sealed::Instance for crate::mcu::peripherals::TIM1 {
    fn id() -> TimerId {
        TimerId::TIM1
    }

    #[inline]
    fn enable_channel_output(en: bool) {
        unsafe { pac::TIM1::PTR.as_ref().unwrap() }
            .bdtr
            .modify(|_, w| w.moe().bit(en));
    }

    #[inline]
    fn set_cms(cms: CenterAlignedMode) {
        unsafe { pac::TIM1::PTR.as_ref().unwrap() }
            .cr1
            .modify(|_, w| unsafe { w.cms().bits(cms as u8) });
    }

    #[inline]
    fn set_dir(dir: CountDirection) {
        unsafe { pac::TIM1::PTR.as_ref().unwrap() }
            .cr1
            .modify(|_, w| w.dir().bit(dir == CountDirection::Down));
    }

    #[inline]
    fn enable_single_mode(en: bool) {
        unsafe { pac::TIM1::PTR.as_ref().unwrap() }
            .cr1
            .modify(|_, w| w.opm().bit(en));
    }

    #[inline]
    fn set_repetition(rep: u8) {
        unsafe { pac::TIM1::PTR.as_ref().unwrap() }
            .rcr
            .write(|w| unsafe { w.rep().bits(rep) });
    }

    #[allow(clippy::too_many_arguments)]
    fn config_break_deadtime(
        dead_time: DeadTime,
        lock_level: LockLevel,
        off_state_run: OffState,
        off_state_idle: OffState,
        break_polarity: BreakPolarity,
        break_enable: bool,
        automatic_output: bool,
    ) {
        unsafe { pac::TIM1::PTR.as_ref().unwrap() }.bdtr.modify(|_, w| unsafe {
            w.dtg()
                .bits(dead_time.value())
                .lock()
                .bits(lock_level as u8)
                .ossr()
                .bit(off_state_run as u8 != 0)
                .ossi()
                .bit(off_state_idle as u8 != 0)
                .bke()
                .bit(break_enable)
                .bkp()
                .bit(break_polarity as u8 != 0)
                .aoe()
                .bit(automatic_output)
                .moe()
                .bit(true)
        });
    }

    fn set_encoder_mode(mode: EncoderMode) {
        let b = unsafe { pac::TIM1::PTR.as_ref().unwrap() };
        b.ccmr1_input()
            .modify(|_, w| unsafe { w.cc1s().bits(1).cc2s().bits(1) });
        b.ccer.modify(|_, w| w.cc1p().clear_bit().cc1np().clear_bit());
        b.ccer.modify(|_, w| w.cc2p().clear_bit().cc2np().clear_bit());
        b.smcr.modify(|_, w| unsafe { w.sms().bits(mode as u8) });
    }

    #[inline]
    fn set_slave_mode(mode: SlaveMode) {
        unsafe { pac::TIM1::PTR.as_ref().unwrap() }
            .smcr
            .modify(|_, w| unsafe { w.sms().bits(mode as u8) });
    }

    #[inline]
    fn set_trigger_selection(ts: TriggerSelection) {
        unsafe { pac::TIM1::PTR.as_ref().unwrap() }
            .smcr
            .modify(|_, w| unsafe { w.ts().bits(ts as u8) });
    }

    #[inline]
    fn set_master_slave_mode(en: bool) {
        unsafe { pac::TIM1::PTR.as_ref().unwrap() }
            .smcr
            .modify(|_, w| w.msm().bit(en));
    }

    #[inline]
    fn set_ocref_clear_source(source: OcRefClearSource) {
        unsafe { pac::TIM1::PTR.as_ref().unwrap() }
            .smcr
            .modify(|_, w| w.occs().bit(source as u8 != 0));
    }

    #[inline]
    fn set_master_output_trigger(trigger: MasterOutputTrigger) {
        unsafe { pac::TIM1::PTR.as_ref().unwrap() }
            .cr2
            .modify(|_, w| unsafe { w.mms().bits(trigger as u8) });
    }
}

//////////////////////// TIM3：无刹车/重复计数 ////////////////////////

impl sealed::Instance for crate::mcu::peripherals::TIM3 {
    fn id() -> TimerId {
        TimerId::TIM3
    }

    #[inline]
    fn set_cms(cms: CenterAlignedMode) {
        unsafe { pac::TIM3::PTR.as_ref().unwrap() }
            .cr1
            .modify(|_, w| unsafe { w.cms().bits(cms as u8) });
    }

    #[inline]
    fn set_dir(dir: CountDirection) {
        unsafe { pac::TIM3::PTR.as_ref().unwrap() }
            .cr1
            .modify(|_, w| w.dir().bit(dir == CountDirection::Down));
    }

    #[inline]
    fn enable_single_mode(en: bool) {
        unsafe { pac::TIM3::PTR.as_ref().unwrap() }
            .cr1
            .modify(|_, w| w.opm().bit(en));
    }

    fn set_encoder_mode(mode: EncoderMode) {
        let b = unsafe { pac::TIM3::PTR.as_ref().unwrap() };
        b.ccmr1_input()
            .modify(|_, w| unsafe { w.cc1s().bits(1).cc2s().bits(1) });
        b.ccer.modify(|_, w| w.cc1p().clear_bit());
        b.ccer.modify(|_, w| w.cc2p().clear_bit());
        b.smcr.modify(|_, w| unsafe { w.sms().bits(mode as u8) });
    }

    #[inline]
    fn set_slave_mode(mode: SlaveMode) {
        unsafe { pac::TIM3::PTR.as_ref().unwrap() }
            .smcr
            .modify(|_, w| unsafe { w.sms().bits(mode as u8) });
    }

    #[inline]
    fn set_trigger_selection(ts: TriggerSelection) {
        unsafe { pac::TIM3::PTR.as_ref().unwrap() }
            .smcr
            .modify(|_, w| unsafe { w.ts().bits(ts as u8) });
    }

    #[inline]
    fn set_master_slave_mode(en: bool) {
        unsafe { pac::TIM3::PTR.as_ref().unwrap() }
            .smcr
            .modify(|_, w| w.msm().bit(en));
    }

    #[inline]
    fn set_ocref_clear_source(source: OcRefClearSource) {
        unsafe { pac::TIM3::PTR.as_ref().unwrap() }
            .smcr
            .modify(|_, w| w.occs().bit(source as u8 != 0));
    }

    #[inline]
    fn set_master_output_trigger(trigger: MasterOutputTrigger) {
        unsafe { pac::TIM3::PTR.as_ref().unwrap() }
            .cr2
            .modify(|_, w| unsafe { w.mms().bits(trigger as u8) });
    }
}

//////////////////////// TIM16：能力覆写（单通道 + 刹车 + 重复计数 + 单脉冲） ////////////////////////

impl sealed::Instance for crate::mcu::peripherals::TIM16 {
    fn id() -> TimerId {
        TimerId::TIM16
    }

    #[inline]
    fn enable_channel_output(en: bool) {
        Self::tim16_block()
            .bdtr
            .modify(|_, w| w.moe().bit(en));
    }

    #[inline]
    fn enable_single_mode(en: bool) {
        Self::tim16_block()
            .cr1
            .modify(|_, w| w.opm().bit(en));
    }

    #[inline]
    fn set_repetition(rep: u8) {
        Self::tim16_block()
            .rcr
            .write(|w| unsafe { w.rep().bits(rep) });
    }

    #[allow(clippy::too_many_arguments)]
    fn config_break_deadtime(
        dead_time: DeadTime,
        lock_level: LockLevel,
        off_state_run: OffState,
        off_state_idle: OffState,
        break_polarity: BreakPolarity,
        break_enable: bool,
        automatic_output: bool,
    ) {
        Self::tim16_block().bdtr.modify(|_, w| unsafe {
            w.dtg()
                .bits(dead_time.value())
                .lock()
                .bits(lock_level as u8)
                .ossr()
                .bit(off_state_run as u8 != 0)
                .ossi()
                .bit(off_state_idle as u8 != 0)
                .bke()
                .bit(break_enable)
                .bkp()
                .bit(break_polarity as u8 != 0)
                .aoe()
                .bit(automatic_output)
                .moe()
                .bit(true)
        });
    }
}

//////////////////////// TIM17：与 TIM16 完全一致 ////////////////////////

impl sealed::Instance for crate::mcu::peripherals::TIM17 {
    fn id() -> TimerId {
        TimerId::TIM17
    }

    #[inline]
    fn enable_channel_output(en: bool) {
        Self::tim16_block().bdtr.modify(|_, w| w.moe().bit(en));
    }

    #[inline]
    fn enable_single_mode(en: bool) {
        Self::tim16_block().cr1.modify(|_, w| w.opm().bit(en));
    }

    #[inline]
    fn set_repetition(rep: u8) {
        Self::tim16_block().rcr.write(|w| unsafe { w.rep().bits(rep) });
    }

    #[allow(clippy::too_many_arguments)]
    fn config_break_deadtime(
        dead_time: DeadTime,
        lock_level: LockLevel,
        off_state_run: OffState,
        off_state_idle: OffState,
        break_polarity: BreakPolarity,
        break_enable: bool,
        automatic_output: bool,
    ) {
        Self::tim16_block().bdtr.modify(|_, w| unsafe {
            w.dtg()
                .bits(dead_time.value())
                .lock()
                .bits(lock_level as u8)
                .ossr()
                .bit(off_state_run as u8 != 0)
                .ossi()
                .bit(off_state_idle as u8 != 0)
                .bke()
                .bit(break_enable)
                .bkp()
                .bit(break_polarity as u8 != 0)
                .aoe()
                .bit(automatic_output)
                .moe()
                .bit(true)
        });
    }
}

//////////////////////// TIM14：能力覆写（仅 TI1 重映射） ////////////////////////

impl sealed::Instance for crate::mcu::peripherals::TIM14 {
    fn id() -> TimerId {
        TimerId::TIM14
    }

    #[inline]
    fn set_ti1_remap(remap: Ti1Remap) {
        unsafe { pac::TIM14::PTR.as_ref().unwrap() }
            .or
            .modify(|_, w| unsafe { w.ti1_rmp().bits(remap as u8) });
    }
}

//////////////////////// 能力标记绑定 ////////////////////////

use super::capability::*;

impl ChannelCount for crate::mcu::peripherals::TIM1 {
    const CHANNELS: usize = 4;
}
impl ChannelCount for crate::mcu::peripherals::TIM3 {
    const CHANNELS: usize = 4;
}
impl ChannelCount for crate::mcu::peripherals::TIM14 {
    const CHANNELS: usize = 1;
}
impl ChannelCount for crate::mcu::peripherals::TIM16 {
    const CHANNELS: usize = 1;
}
impl ChannelCount for crate::mcu::peripherals::TIM17 {
    const CHANNELS: usize = 1;
}

macro_rules! impl_channels {
    ($peripheral:ident, [$($n:literal),*]) => {
        $(impl HasChannel<$n> for crate::mcu::peripherals::$peripheral {})*
    };
}

impl_channels!(TIM1, [1, 2, 3, 4]);
impl_channels!(TIM3, [1, 2, 3, 4]);
impl_channels!(TIM14, [1]);
impl_channels!(TIM16, [1]);
impl_channels!(TIM17, [1]);

// 互补输出 / 刹车 / 重复计数：TIM1、TIM16、TIM17
macro_rules! impl_break_caps {
    ($($peripheral:ident),*) => {
        $(
            impl WithComplementary for crate::mcu::peripherals::$peripheral {}
            impl WithBreak for crate::mcu::peripherals::$peripheral {}
            impl WithRepetition for crate::mcu::peripherals::$peripheral {}
        )*
    };
}
impl_break_caps!(TIM1, TIM16, TIM17);

// 编码器 / 从模式 / 中心对齐 / 主模式 TRGO：TIM1、TIM3
macro_rules! impl_slave_caps {
    ($($peripheral:ident),*) => {
        $(
            impl WithEncoder for crate::mcu::peripherals::$peripheral {}
            impl WithCenterAlign for crate::mcu::peripherals::$peripheral {}
            impl WithMasterMode for crate::mcu::peripherals::$peripheral {}
        )*
    };
}
impl_slave_caps!(TIM1, TIM3);

// 单脉冲：TIM1、TIM3、TIM16、TIM17
impl WithOnePulse for crate::mcu::peripherals::TIM1 {}
impl WithOnePulse for crate::mcu::peripherals::TIM3 {}
impl WithOnePulse for crate::mcu::peripherals::TIM16 {}
impl WithOnePulse for crate::mcu::peripherals::TIM17 {}

// TI1 重映射：仅 TIM14
impl WithTi1Remap for crate::mcu::peripherals::TIM14 {}

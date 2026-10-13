//! Unified APB timer instance layer.
//!
//! Design notes:
//! 1. All PY32 TIM peripherals (TIM1 / TIM3 / TIM14 / TIM16 / TIM17) share the **same
//!    register layout for the common registers** (CR1, CR2, DIER, SR, EGR, CCER, CNT,
//!    PSC, ARR, CCR1, CCMR1/CCMR2 views). Instead of duplicating every basic operation
//!    in a `match Self::id()` arm per timer (which bloats the binary), we cast the
//!    peripheral base address to the richest `pac::tim1::RegisterBlock` via [`Instance::block`]
//!    and operate on that single typed block. This mirrors the CMSIS `TIM_TypeDef *` cast.
//! 2. Timer-specific registers that do **not** exist on every timer (CR2/OIS, SMCR, RCR,
//!    BDTR) are only ever accessed from the capability-gated `Instance` overrides, which are
//!    only instantiated for the timers that actually have them — so the base cast never
//!    touches a non-existent register. Methods with genuinely different field sets
//!    (channel config / event mapping) keep a `match Self::id()` but reuse `block()`.
//! 3. Timer-specific capabilities (break, repetition counter, encoder, center-aligned mode,
//!    one-pulse, TRGO, TI1 remap) default to no-ops and are only overridden by instances
//!    that have the hardware. These capabilities are gated at **compile time** by the marker
//!    traits in [`crate::timer::capability`], so users never call a no-op - instead the code
//!    fails to compile.

use super::types::*;
use crate::clock::peripheral::{PeripheralClockIndex, PeripheralIdToClockIndex};
use crate::clock::timer_pclk;
use crate::pac::{self, interrupt};

pub(crate) mod sealed {
    use super::*;

    pub trait Instance {
        /// Timer index.
        fn id() -> TimerId;

        /// Common timer register block.
        ///
        /// Casts the peripheral base address to the richest layout
        /// (`pac::tim1::RegisterBlock`) and only ever touches registers that physically
        /// exist on the concrete timer. See the module docs for the safety argument.
        #[inline]
        fn block() -> &'static pac::tim1::RegisterBlock {
            let base = match Self::id() {
                TimerId::TIM1 => pac::TIM1::PTR as *const pac::tim1::RegisterBlock,
                TimerId::TIM3 => pac::TIM3::PTR as *const pac::tim1::RegisterBlock,
                TimerId::TIM14 => pac::TIM14::PTR as *const pac::tim1::RegisterBlock,
                TimerId::TIM16 => pac::TIM16::PTR as *const pac::tim1::RegisterBlock,
                TimerId::TIM17 => pac::TIM17::PTR as *const pac::tim1::RegisterBlock,
            };
            unsafe { &*base }
        }

        /// Return the timer peripheral clock.
        #[inline]
        fn get_time_pclk() -> u32 {
            timer_pclk()
        }

        /// Start counting.
        #[inline]
        fn start() {
            Self::block().cr1.modify(|_, w| w.cen().set_bit());
        }

        /// Stop counting.
        #[inline]
        fn stop() {
            Self::block().cr1.modify(|_, w| w.cen().clear_bit());
        }

        /// Set the prescaler.
        #[inline]
        fn set_prescaler(prescaler: u16) {
            Self::block()
                .psc
                .write(|w| unsafe { w.psc().bits(prescaler) });
        }

        /// Set the period counter value (ARR, preserving other bits).
        #[inline]
        fn set_period_cnt(cnt: u16) {
            Self::block()
                .arr
                .modify(|_, w| unsafe { w.arr().bits(cnt) });
        }

        /// Set the current counter value.
        #[inline]
        fn set_cnt(cnt: u16) {
            Self::block()
                .cnt
                .write(|w| unsafe { w.cnt().bits(cnt) });
        }

        /// Read the current counter value.
        #[inline]
        fn get_cnt() -> u16 {
            Self::block().cnt.read().cnt().bits()
        }

        /// Set the auto-reload value (ARR).
        #[inline]
        fn set_auto_reload(load: u16) {
            Self::block()
                .arr
                .write(|w| unsafe { w.arr().bits(load) });
        }

        /// Read the auto-reload value.
        #[inline]
        fn get_reload() -> u16 {
            Self::block().arr.read().arr().bits()
        }

        /// Auto-reload preload enable bit (ARPE).
        #[inline]
        fn enable_auto_reload_buff(en: bool) {
            Self::block().cr1.modify(|_, w| w.arpe().bit(en));
        }

        /// Return the current counting frequency (CK_CNT).
        #[inline]
        fn counter_frequency() -> u32 {
            // CK_CNT = fCK_PSC / (PSC[15:0] + 1)
            let psc: u32 = Self::block().psc.read().bits();
            timer_pclk() / (psc + 1)
        }

        /// Set the clock division factor tDTS (CR1.CKD).
        #[inline]
        fn set_clock_division(div: ClockDiv) {
            Self::block()
                .cr1
                .modify(|_, w| unsafe { w.ckd().bits(div as u8) });
        }

        /// Set the channel input/output type (CCxS).
        fn set_channel_type(channel: Channel, channel_type: ChannelType) {
            let v = channel_type as u8;
            let b = Self::block();
            match Self::id() {
                TimerId::TIM1 | TimerId::TIM3 => match channel {
                    Channel::CH1 => b.ccmr1_output().modify(|_, w| unsafe { w.cc1s().bits(v) }),
                    Channel::CH2 => b.ccmr1_output().modify(|_, w| unsafe { w.cc2s().bits(v) }),
                    Channel::CH3 => b.ccmr2_output().modify(|_, w| unsafe { w.cc3s().bits(v) }),
                    Channel::CH4 => b.ccmr2_output().modify(|_, w| unsafe { w.cc4s().bits(v) }),
                },
                // Single-channel timers: only CH1 is valid.
                TimerId::TIM14 | TimerId::TIM16 | TimerId::TIM17 => {
                    if matches!(channel, Channel::CH1) {
                        b.ccmr1_output().modify(|_, w| unsafe { w.cc1s().bits(v) });
                    }
                }
            }
        }

        /// Set the channel output-compare mode (OCxM / OCxCE / OCxFE / OCxPE).
        #[allow(clippy::too_many_arguments)]
        fn set_channel_output_config(
            channel: Channel,
            mode: ChannelMode,
            clear: bool,
            fast: bool,
            preload: bool,
        ) {
            let m = mode as u8;
            let b = Self::block();
            match Self::id() {
                TimerId::TIM1 | TimerId::TIM3 => match channel {
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
                },
                // TIM14 / TIM16 have no OC1CE bit.
                TimerId::TIM14 | TimerId::TIM16 | TimerId::TIM17 => {
                    if matches!(channel, Channel::CH1) {
                        b.ccmr1_output().modify(|_, w| unsafe {
                            w.oc1m().bits(m).oc1fe().bit(fast).oc1pe().bit(preload)
                        });
                    }
                }
            }
        }

        /// Set the channel capture/compare value (CCRx).
        fn set_channel_compare(channel: Channel, ccr: u16) {
            let b = Self::block();
            match channel {
                Channel::CH1 => b.ccr1.write(|w| unsafe { w.ccr1().bits(ccr) }),
                Channel::CH2 => b.ccr2.write(|w| unsafe { w.ccr2().bits(ccr) }),
                Channel::CH3 => b.ccr3.write(|w| unsafe { w.ccr3().bits(ccr) }),
                Channel::CH4 => b.ccr4.write(|w| unsafe { w.ccr4().bits(ccr) }),
            }
        }

        /// Read the channel capture value (CCRx).
        fn get_channel_capture(channel: Channel) -> u16 {
            let b = Self::block();
            match channel {
                Channel::CH1 => b.ccr1.read().ccr1().bits(),
                Channel::CH2 => b.ccr2.read().ccr2().bits(),
                Channel::CH3 => b.ccr3.read().ccr3().bits(),
                Channel::CH4 => b.ccr4.read().ccr4().bits(),
            }
        }

        /// Set the output channel active polarity / output enable / idle state (OIS).
        fn set_channel_output_effective_level(
            channel: Channel,
            channel_output: ChannelOutput,
            polarity: bool,
            state: bool,
            idle: bool,
        ) {
            let b = Self::block();
            match Self::id() {
                TimerId::TIM1 => match (channel, channel_output) {
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
                    // TIM1 has no CH4N.
                    (Channel::CH4, ChannelOutput::N) => {}
                },
                // TIM3 has no complementary output and no OIS.
                TimerId::TIM3 => match channel {
                    Channel::CH1 => b.ccer.modify(|_, w| w.cc1p().bit(!polarity).cc1e().bit(state)),
                    Channel::CH2 => b.ccer.modify(|_, w| w.cc2p().bit(!polarity).cc2e().bit(state)),
                    Channel::CH3 => b.ccer.modify(|_, w| w.cc3p().bit(!polarity).cc3e().bit(state)),
                    Channel::CH4 => b.ccer.modify(|_, w| w.cc4p().bit(!polarity).cc4e().bit(state)),
                },
                // TIM14 has no CR2 (no OIS), only CH1P.
                TimerId::TIM14 => {
                    if matches!((channel, channel_output), (Channel::CH1, ChannelOutput::P)) {
                        b.ccer
                            .modify(|_, w| w.cc1p().bit(!polarity).cc1e().bit(state));
                    }
                }
                // TIM16 / TIM17 have only CH1, with complementary output and OIS1/OIS1N.
                TimerId::TIM16 | TimerId::TIM17 => match (channel, channel_output) {
                    (Channel::CH1, ChannelOutput::P) => {
                        b.ccer.modify(|_, w| w.cc1p().bit(!polarity).cc1e().bit(state));
                        b.cr2.modify(|_, w| w.ois1().bit(idle));
                    }
                    (Channel::CH1, ChannelOutput::N) => {
                        b.ccer.modify(|_, w| w.cc1np().bit(!polarity).cc1ne().bit(state));
                        b.cr2.modify(|_, w| w.ois1n().bit(idle));
                    }
                    _ => {}
                },
            }
        }

        /// Enable / disable a channel output independently (CCxE / CCxNE).
        fn set_enable_channel(channel: Channel, channel_output: ChannelOutput, en: bool) {
            let b = Self::block();
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

        /// Configure the channel as input capture.
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
            let b = Self::block();
            match Self::id() {
                TimerId::TIM1 => match channel {
                    // Note: TIM1's CH1 input prescaler field is named icpsc (no digit).
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
                    // CH4 has no complementary input.
                    Channel::CH4 => {
                        b.ccmr2_input().modify(|_, w| unsafe {
                            w.cc4s().bits(sel).ic4psc().bits(psc).ic4f().bits(f)
                        });
                        b.ccer.modify(|_, w| w.cc4p().bit(p));
                    }
                },
                TimerId::TIM3 => match channel {
                    Channel::CH1 => {
                        b.ccmr1_input().modify(|_, w| unsafe {
                            w.cc1s().bits(sel).icpsc().bits(psc).ic1f().bits(f)
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
                },
                // Single channel, only cc1p (TIM14 has no CC1NP).
                TimerId::TIM14 => {
                    if matches!(channel, Channel::CH1) {
                        b.ccmr1_input()
                            .modify(|_, w| unsafe { w.cc1s().bits(sel).icpsc().bits(psc).ic1f().bits(f) });
                        b.ccer.modify(|_, w| w.cc1p().bit(p));
                    }
                }
                TimerId::TIM16 | TimerId::TIM17 => {
                    if matches!(channel, Channel::CH1) {
                        b.ccmr1_input().modify(|_, w| unsafe {
                            w.cc1s().bits(sel).icpsc().bits(psc).ic1f().bits(f)
                        });
                        b.ccer.modify(|_, w| w.cc1p().bit(p).cc1np().bit(np));
                    }
                }
            }
        }

        /// Input capture polarity -> CCER CCxP / CCxNP.
        #[inline]
        fn input_polarity_bits(polarity: InputCapturePolarity) -> (bool, bool) {
            match polarity {
                InputCapturePolarity::Rising => (false, false),
                InputCapturePolarity::Falling => (true, false),
                InputCapturePolarity::BothEdge => (true, true),
            }
        }

        /// Generate an event in software (EGR).
        fn triggle(signal: Triggle) {
            let b = Self::block();
            match signal {
                Triggle::UG => b.egr.write(|w| w.ug().set_bit()),
                Triggle::CC1G => b.egr.write(|w| w.cc1g().set_bit()),
                Triggle::CC2G => b.egr.write(|w| w.cc2g().set_bit()),
                Triggle::CC3G => b.egr.write(|w| w.cc3g().set_bit()),
                Triggle::CC4G => b.egr.write(|w| w.cc4g().set_bit()),
                Triggle::COMG => b.egr.write(|w| w.comg().set_bit()),
                Triggle::TG => b.egr.write(|w| w.tg().set_bit()),
                Triggle::BG => b.egr.write(|w| w.bg().set_bit()),
            }
        }

        /// Event flags (SR).
        fn event_flag(event: Event) -> bool {
            let b = Self::block();
            let sr = b.sr.read();
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

        /// Event interrupt enable / mask (DIER).
        fn event_config(event: Event, en: bool) {
            let b = Self::block();
            match Self::id() {
                TimerId::TIM1 => b.dier.modify(|_, w| match event {
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
                // TIM3 has no COMIF / BIF.
                TimerId::TIM3 => b.dier.modify(|_, w| match event {
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
                }),
                // TIM14 DIER has no CC1DE; over-capture is enabled together with CC1IE.
                TimerId::TIM14 => b.dier.modify(|_, w| match event {
                    Event::UIF => w.uie().bit(en),
                    Event::CC1IF | Event::CC1OF => w.cc1ie().bit(en),
                    _ => w,
                }),
                // TIM16: UIF / CC1IF / COMIF / BIF / CC1OF.
                TimerId::TIM16 | TimerId::TIM17 => b.dier.modify(|_, w| match event {
                    Event::UIF => w.uie().bit(en),
                    Event::CC1IF | Event::CC1OF => w.cc1ie().bit(en),
                    Event::COMIF => w.comie().bit(en),
                    Event::BIF => w.bie().bit(en),
                    _ => w,
                }),
            }
        }

        /// Clear event flags (SR).
        fn event_clear(event: Event) {
            let b = Self::block();
            match Self::id() {
                TimerId::TIM1 => b.sr.modify(|_, w| match event {
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
                TimerId::TIM3 => b.sr.modify(|_, w| match event {
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
                TimerId::TIM14 => b.sr.modify(|_, w| match event {
                    Event::UIF => w.uif().clear_bit(),
                    Event::CC1IF => w.cc1if().clear_bit(),
                    Event::CC1OF => w.cc1of().clear_bit(),
                    _ => w,
                }),
                TimerId::TIM16 | TimerId::TIM17 => b.sr.modify(|_, w| match event {
                    Event::UIF => w.uif().clear_bit(),
                    Event::CC1IF => w.cc1if().clear_bit(),
                    Event::COMIF => w.comif().clear_bit(),
                    Event::BIF => w.bif().clear_bit(),
                    Event::CC1OF => w.cc1of().clear_bit(),
                    _ => w,
                }),
            }
        }

        /// Compute (PSC, RCR, ARR) from microseconds.
        #[inline]
        fn micros_to_compute_with_rep(micros: u64) -> (u16, u8, u16) {
            let ticks = micros * Self::get_time_pclk() as u64 / 1_000_000;
            let psc = ticks / (1u64 << 24);
            let count = ticks / (psc + 1);
            let rep = count / (1u64 << 16);
            let arr = count / (rep + 1);
            (psc as u16, rep as u8, arr as u16)
        }

        /// Compute (PSC, RCR, ARR) from nanoseconds.
        #[inline]
        fn nanosecond_to_compute_with_rep(nano: u64) -> (u16, u8, u16) {
            let ticks = nano * Self::get_time_pclk() as u64 / 1_000_000_000;
            let psc = ticks / (1u64 << 24);
            let count = ticks / (psc + 1);
            let rep = count / (1u64 << 16);
            let arr = count / (rep + 1);
            (psc as u16, rep as u8, arr as u16)
        }

        //// All default to no-ops. Only instances with the corresponding hardware override them,
        //// and the high-level API gates them at compile time via capability traits, so users never call a no-op.

        /// Enable the main output (BDTR.MOE), TIM1 / TIM16 only.
        #[inline]
        fn enable_channel_output(_en: bool) {}

        /// Set the center-aligned mode, TIM1 / TIM3 only.
        #[inline]
        fn set_cms(_cms: CenterAlignedMode) {}

        /// Set the count direction, TIM1 / TIM3 only.
        #[inline]
        fn set_dir(_dir: CountDirection) {}

        /// One-pulse mode (OPM), TIM1 / TIM3 / TIM16.
        #[inline]
        fn enable_single_mode(_en: bool) {}

        /// Repetition counter (RCR), TIM1 / TIM16 only.
        #[inline]
        fn set_repetition(_rep: u8) {}

        /// Break and dead-time (BDTR), TIM1 / TIM16 only.
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

        /// Encoder mode (SMCR), TIM1 / TIM3 only.
        #[inline]
        fn set_encoder_mode(_mode: EncoderMode) {}

        /// Slave mode (SMCR.SMS), TIM1 / TIM3 only.
        #[inline]
        fn set_slave_mode(_mode: SlaveMode) {}

        /// Trigger input selection (SMCR.TS), TIM1 / TIM3 only.
        #[inline]
        fn set_trigger_selection(_ts: TriggerSelection) {}

        /// Master/slave mode synchronization (SMCR.MSM), TIM1 / TIM3 only.
        #[inline]
        fn set_master_slave_mode(_en: bool) {}

        /// OCREF clear source (SMCR.OCCS), TIM1 / TIM3 only.
        #[inline]
        fn set_ocref_clear_source(_source: OcRefClearSource) {}

        /// Master mode trigger output TRGO (CR2.MMS), TIM1 / TIM3 only.
        #[inline]
        fn set_master_output_trigger(_trigger: MasterOutputTrigger) {}

        /// TI1 input remap (OR), TIM14 only.
        #[inline]
        fn set_ti1_remap(_remap: Ti1Remap) {}
    }
}

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
    /// All NVIC interrupt vectors occupied by this timer.
    ///
    /// TIM1 has **two** interrupt vectors (BRK_UP_TRG_COM and CC); all others have one.
    pub fn interrupts(&self) -> &'static [interrupt] {
        match *self {
            TimerId::TIM1 => &[interrupt::TIM1_BRK_UP_TRG_COM, interrupt::TIM1_CC],
            TimerId::TIM3 => &[interrupt::TIM3],
            TimerId::TIM14 => &[interrupt::TIM14],
            TimerId::TIM16 => &[interrupt::TIM16],
            TimerId::TIM17 => &[interrupt::TIM17],
        }
    }

    /// Enable all interrupt vectors of this timer (NVIC unmask).
    #[inline]
    pub fn enable_interrupts(&self) {
        for irq in self.interrupts() {
            unsafe { cortex_m::peripheral::NVIC::unmask(*irq) }
        }
    }

    /// Disable all interrupt vectors of this timer (NVIC mask).
    #[inline]
    pub fn disable_interrupts(&self) {
        for irq in self.interrupts() {
            cortex_m::peripheral::NVIC::mask(*irq)
        }
    }
}

impl sealed::Instance for crate::mcu::peripherals::TIM1 {
    fn id() -> TimerId {
        TimerId::TIM1
    }

    #[inline]
    fn enable_channel_output(en: bool) {
        Self::block().bdtr.modify(|_, w| w.moe().bit(en));
    }

    #[inline]
    fn set_cms(cms: CenterAlignedMode) {
        Self::block().cr1.modify(|_, w| unsafe { w.cms().bits(cms as u8) });
    }

    #[inline]
    fn set_dir(dir: CountDirection) {
        Self::block()
            .cr1
            .modify(|_, w| w.dir().bit(dir == CountDirection::Down));
    }

    #[inline]
    fn enable_single_mode(en: bool) {
        Self::block().cr1.modify(|_, w| w.opm().bit(en));
    }

    #[inline]
    fn set_repetition(rep: u8) {
        Self::block().rcr.write(|w| unsafe { w.rep().bits(rep) });
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
        Self::block().bdtr.modify(|_, w| unsafe {
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
        let b = Self::block();
        b.ccmr1_input()
            .modify(|_, w| unsafe { w.cc1s().bits(1).cc2s().bits(1) });
        b.ccer.modify(|_, w| w.cc1p().clear_bit().cc1np().clear_bit());
        b.ccer.modify(|_, w| w.cc2p().clear_bit().cc2np().clear_bit());
        b.smcr.modify(|_, w| unsafe { w.sms().bits(mode as u8) });
    }

    #[inline]
    fn set_slave_mode(mode: SlaveMode) {
        Self::block()
            .smcr
            .modify(|_, w| unsafe { w.sms().bits(mode as u8) });
    }

    #[inline]
    fn set_trigger_selection(ts: TriggerSelection) {
        Self::block()
            .smcr
            .modify(|_, w| unsafe { w.ts().bits(ts as u8) });
    }

    #[inline]
    fn set_master_slave_mode(en: bool) {
        Self::block().smcr.modify(|_, w| w.msm().bit(en));
    }

    #[inline]
    fn set_ocref_clear_source(source: OcRefClearSource) {
        Self::block()
            .smcr
            .modify(|_, w| w.occs().bit(source as u8 != 0));
    }

    #[inline]
    fn set_master_output_trigger(trigger: MasterOutputTrigger) {
        Self::block()
            .cr2
            .modify(|_, w| unsafe { w.mms().bits(trigger as u8) });
    }
}

impl sealed::Instance for crate::mcu::peripherals::TIM3 {
    fn id() -> TimerId {
        TimerId::TIM3
    }

    #[inline]
    fn set_cms(cms: CenterAlignedMode) {
        Self::block().cr1.modify(|_, w| unsafe { w.cms().bits(cms as u8) });
    }

    #[inline]
    fn set_dir(dir: CountDirection) {
        Self::block()
            .cr1
            .modify(|_, w| w.dir().bit(dir == CountDirection::Down));
    }

    #[inline]
    fn enable_single_mode(en: bool) {
        Self::block().cr1.modify(|_, w| w.opm().bit(en));
    }

    fn set_encoder_mode(mode: EncoderMode) {
        let b = Self::block();
        b.ccmr1_input()
            .modify(|_, w| unsafe { w.cc1s().bits(1).cc2s().bits(1) });
        b.ccer.modify(|_, w| w.cc1p().clear_bit());
        b.ccer.modify(|_, w| w.cc2p().clear_bit());
        b.smcr.modify(|_, w| unsafe { w.sms().bits(mode as u8) });
    }

    #[inline]
    fn set_slave_mode(mode: SlaveMode) {
        Self::block()
            .smcr
            .modify(|_, w| unsafe { w.sms().bits(mode as u8) });
    }

    #[inline]
    fn set_trigger_selection(ts: TriggerSelection) {
        Self::block()
            .smcr
            .modify(|_, w| unsafe { w.ts().bits(ts as u8) });
    }

    #[inline]
    fn set_master_slave_mode(en: bool) {
        Self::block().smcr.modify(|_, w| w.msm().bit(en));
    }

    #[inline]
    fn set_ocref_clear_source(source: OcRefClearSource) {
        Self::block()
            .smcr
            .modify(|_, w| w.occs().bit(source as u8 != 0));
    }

    #[inline]
    fn set_master_output_trigger(trigger: MasterOutputTrigger) {
        Self::block()
            .cr2
            .modify(|_, w| unsafe { w.mms().bits(trigger as u8) });
    }
}

impl sealed::Instance for crate::mcu::peripherals::TIM16 {
    fn id() -> TimerId {
        TimerId::TIM16
    }

    #[inline]
    fn enable_channel_output(en: bool) {
        Self::block().bdtr.modify(|_, w| w.moe().bit(en));
    }

    #[inline]
    fn enable_single_mode(en: bool) {
        Self::block().cr1.modify(|_, w| w.opm().bit(en));
    }

    #[inline]
    fn set_repetition(rep: u8) {
        Self::block().rcr.write(|w| unsafe { w.rep().bits(rep) });
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
        Self::block().bdtr.modify(|_, w| unsafe {
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

impl sealed::Instance for crate::mcu::peripherals::TIM17 {
    fn id() -> TimerId {
        TimerId::TIM17
    }

    #[inline]
    fn enable_channel_output(en: bool) {
        Self::block().bdtr.modify(|_, w| w.moe().bit(en));
    }

    #[inline]
    fn enable_single_mode(en: bool) {
        Self::block().cr1.modify(|_, w| w.opm().bit(en));
    }

    #[inline]
    fn set_repetition(rep: u8) {
        Self::block().rcr.write(|w| unsafe { w.rep().bits(rep) });
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
        Self::block().bdtr.modify(|_, w| unsafe {
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

// Complementary output / break / repetition counter: TIM1, TIM16, TIM17
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

// Encoder / slave mode / center-aligned / master-mode TRGO: TIM1, TIM3
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

// One-pulse: TIM1, TIM3, TIM16, TIM17
impl WithOnePulse for crate::mcu::peripherals::TIM1 {}
impl WithOnePulse for crate::mcu::peripherals::TIM3 {}
impl WithOnePulse for crate::mcu::peripherals::TIM16 {}
impl WithOnePulse for crate::mcu::peripherals::TIM17 {}

// TI1 remap: TIM14 only
impl WithTi1Remap for crate::mcu::peripherals::TIM14 {}

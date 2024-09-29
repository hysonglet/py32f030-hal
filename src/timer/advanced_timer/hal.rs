pub(crate) mod sealed {
    use super::super::*;
    use crate::clock::timer_pclk;
    use crate::pac;

    pub trait Instance {
        /// 考虑以后其他单片机可能有多个相同外设
        /// 高级定时器的索引
        fn id() -> AdvancedTimer;

        /// 返回外设的基地址
        #[inline]
        fn block() -> &'static pac::tim1::RegisterBlock {
            match Self::id() {
                AdvancedTimer::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() },
            }
        }

        /// 启动定时
        #[inline]
        fn start() {
            Self::block().cr1.modify(|_, w| w.cen().set_bit())
        }

        /// 停止定时器
        #[inline]
        fn stop() {
            Self::block().cr1.modify(|_, w| w.cen().clear_bit())
        }

        /// 返回定时器外设时钟
        #[inline]
        fn get_time_pclk() -> u32 {
            timer_pclk()
        }

        /// 设置预分频的参数，
        /// 预分频器可以将计数器的时钟按 1 到 65535 之间的任意值分频。它是基于一个（在 TIM3_PSC 寄存器中
        /// 的）16 位寄存器控制的 16 位计数器。因为这个控制寄存器带有缓冲器，它能够在运行时被改变。新的预分频
        /// 器的参数在下一次更新事件到来时被采用。
        #[inline]
        fn set_prescaler(prescaler: u16) {
            Self::block()
                .psc
                .write(|w| unsafe { w.psc().bits(prescaler) })
        }

        /// 设置计数周期值
        #[inline]
        fn set_period_cnt(cnt: u16) {
            Self::block()
                .psc
                .modify(|_, w| unsafe { w.psc().bits(cnt) })
        }

        /// 设置计数值
        #[inline]
        fn set_cnt(cnt: u16) {
            Self::block().cnt.write(|w| unsafe { w.cnt().bits(cnt) })
        }

        /// 获取计数值
        #[inline]
        fn get_cnt() -> u16 {
            Self::block().cnt.read().cnt().bits()
        }

        /// 设置中央对齐模式
        #[inline]
        fn set_cms(cms: CenterAlignedMode) {
            Self::block()
                .cr1
                .modify(|_, w| unsafe { w.cms().bits(cms as u8) });
        }

        /// 设置计数方向
        #[inline]
        fn set_dir(dir: CountDirection) {
            Self::block().cr1.modify(|_, w| w.dir().bit(dir.into()));
        }

        /// 设置自动重载的值
        #[inline]
        fn set_auto_reload(load: u16) {
            Self::block().arr.write(|w| unsafe { w.arr().bits(load) })
        }

        /// 获取重载值
        #[inline]
        fn get_reload() -> u16 {
            Self::block().arr.read().arr().bits()
        }

        /// 单脉冲模式
        /// 0：在发生更新事件时，计数器不停止
        /// 1：在发生下一次更新事件(清除 CEN 位)时，计数器停止。
        #[inline]
        fn enable_single_mode(en: bool) {
            Self::block().cr1.modify(|_, w| w.opm().bit(en))
        }

        /// 自动重装载预装载允许位
        /// 0： TIM1_ARR 寄存器没有缓冲
        /// 1： TIM1_ARR 寄存器被装入缓冲器
        #[inline]
        fn enable_auto_reload_buff(en: bool) {
            Self::block().cr1.modify(|_, w| w.arpe().bit(en))
        }

        /// 返回计数频率
        #[inline]
        fn counter_frequency() -> u32 {
            let psc: u32 = Self::block().psc.read().bits();
            // 计数器的时钟频率（CK_CNT）等于 fCK_PSC/( PSC[15:0]+1)。
            timer_pclk() / (psc + 1)
        }

        /// 设置重复值
        #[inline]
        fn set_repetition(repetition: u8) {
            Self::block()
                .rcr
                .write(|w| unsafe { w.rep().bits(repetition) })
        }

        /// 设置通道输入或输出类型
        fn set_channel_type(channel: Channel, channel_type: ChannelType) {
            let block = Self::block();
            match channel {
                Channel::CH1 => block
                    .ccmr1_output()
                    .modify(|_, w| unsafe { w.cc1s().bits(channel_type as u8) }),
                Channel::CH2 => block
                    .ccmr1_output()
                    .modify(|_, w| unsafe { w.cc2s().bits(channel_type as u8) }),
                Channel::CH3 => block
                    .ccmr2_output()
                    .modify(|_, w| unsafe { w.cc3s().bits(channel_type as u8) }),
                Channel::CH4 => block
                    .ccmr2_output()
                    .modify(|_, w| unsafe { w.cc4s().bits(channel_type as u8) }),
            }
        }

        /// 设置通道输出模式
        fn set_channel_output_config(
            channel: Channel,
            mode: ChannelMode,
            clear: bool,
            fast: bool,
            preload: bool,
        ) {
            let block = Self::block();
            match channel {
                Channel::CH1 => block.ccmr1_output().modify(|_, w| unsafe {
                    w.oc1m()
                        .bits(mode as u8)
                        .oc1ce()
                        .bit(clear)
                        .oc1fe()
                        .bit(fast)
                        .oc1pe()
                        .bit(preload)
                }),
                Channel::CH2 => block.ccmr1_output().modify(|_, w| {
                    unsafe { w.oc2m().bits(mode as u8) }
                        .oc2ce()
                        .bit(clear)
                        .oc2fe()
                        .bit(fast)
                        .oc2pe()
                        .bit(preload)
                }),
                Channel::CH3 => block.ccmr2_output().modify(|_, w| unsafe {
                    w.oc3m()
                        .bits(mode as u8)
                        .oc3ce()
                        .bit(clear)
                        .oc3fe()
                        .bit(fast)
                        .oc3pe()
                        .bit(preload)
                }),
                Channel::CH4 => block.ccmr2_output().modify(|_, w| unsafe {
                    w.oc4m()
                        .bits(mode as u8)
                        .oc4ce()
                        .bit(clear)
                        .oc4fe()
                        .bit(fast)
                        .oc4pe()
                        .bit(preload)
                }),
            }
        }

        /// 使能通道连接到引脚
        #[inline]
        fn enable_channel_output(en: bool) {
            Self::block().bdtr.write(|w| w.moe().bit(en));
        }

        /// 设置通道的捕获/比较值
        fn set_channel_compare(channel: Channel, ccr: u16) {
            let block = Self::block();
            match channel {
                Channel::CH1 => {
                    // defmt::info!("compare: {}", ccr);
                    block.ccr1.write(|w| unsafe { w.ccr1().bits(ccr) })
                }
                Channel::CH2 => block.ccr2.modify(|_, w| unsafe { w.ccr2().bits(ccr) }),
                Channel::CH3 => block.ccr3.modify(|_, w| unsafe { w.ccr3().bits(ccr) }),
                Channel::CH4 => block.ccr4.modify(|_, w| unsafe { w.ccr4().bits(ccr) }),
            }
        }

        /// 返回捕获的值
        fn get_channel_capture(channel: Channel) -> u16 {
            let block = Self::block();
            match channel {
                Channel::CH1 => block.ccr1.read().ccr1().bits(),
                Channel::CH2 => block.ccr2.read().ccr2().bits(),
                Channel::CH3 => block.ccr3.read().ccr3().bits(),
                Channel::CH4 => block.ccr4.read().ccr4().bits(),
            }
        }

        /// 设置输出通道的有效极性，true: 高电平   ，false：低电平
        fn set_channel_output_effective_level(
            channel: Channel,
            channel_output: ChannelOutput,
            polarity: bool,
            state: bool,
            idle: bool,
        ) {
            let block = Self::block();
            match (channel, channel_output) {
                (Channel::CH1, ChannelOutput::P) => {
                    block
                        .ccer
                        .modify(|_, w| w.cc1p().bit(!polarity).cc1e().bit(state));
                    block.cr2.modify(|_, w| w.ois1().bit(idle))
                }
                (Channel::CH2, ChannelOutput::P) => {
                    block
                        .ccer
                        .modify(|_, w| w.cc2p().bit(!polarity).cc2e().bit(state));
                    block.cr2.modify(|_, w| w.ois1n().bit(idle))
                }
                (Channel::CH3, ChannelOutput::P) => {
                    block
                        .ccer
                        .modify(|_, w| w.cc3p().bit(!polarity).cc3e().bit(state));
                    block.cr2.modify(|_, w| w.ois2().bit(idle))
                }
                (Channel::CH4, ChannelOutput::P) => {
                    block
                        .ccer
                        .modify(|_, w| w.cc4p().bit(!polarity).cc4e().bit(state));
                    block.cr2.modify(|_, w| w.ois2n().bit(idle))
                }
                (Channel::CH1, ChannelOutput::N) => {
                    block
                        .ccer
                        .modify(|_, w| w.cc1np().bit(!polarity).cc1ne().bit(state));
                    block.cr2.modify(|_, w| w.ois3().bit(idle))
                }
                (Channel::CH2, ChannelOutput::N) => {
                    block
                        .ccer
                        .modify(|_, w| w.cc2np().bit(!polarity).cc2ne().bit(state));
                    block.cr2.modify(|_, w| w.ois3n().bit(idle))
                }
                (Channel::CH3, ChannelOutput::N) => {
                    block
                        .ccer
                        .modify(|_, w| w.cc3np().bit(!polarity).cc3ne().bit(state));
                    block.cr2.modify(|_, w| w.ois4().bit(idle))
                }
                (Channel::CH4, ChannelOutput::N) => {}
            }
        }

        /// 使能 P 通道
        fn set_enable_channel(channel: Channel, channel_output: ChannelOutput, en: bool) {
            let block = Self::block();
            match (channel, channel_output) {
                (Channel::CH1, ChannelOutput::P) => block.ccer.modify(|_, w| w.cc1e().bit(en)),
                (Channel::CH2, ChannelOutput::P) => block.ccer.modify(|_, w| w.cc2e().bit(en)),
                (Channel::CH3, ChannelOutput::P) => block.ccer.modify(|_, w| w.cc3e().bit(en)),
                (Channel::CH4, ChannelOutput::P) => block.ccer.modify(|_, w| w.cc4e().bit(en)),
                (Channel::CH1, ChannelOutput::N) => block.ccer.modify(|_, w| w.cc1ne().bit(en)),
                (Channel::CH2, ChannelOutput::N) => block.ccer.modify(|_, w| w.cc2ne().bit(en)),
                (Channel::CH3, ChannelOutput::N) => block.ccer.modify(|_, w| w.cc3ne().bit(en)),
                (Channel::CH4, ChannelOutput::N) => {}
            }
        }

        /// 软件方式触发信号
        #[inline]
        fn triggle(signal: Triggle) {
            let egr = &Self::block().egr;
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

        /// 返回事件标志
        #[inline]
        fn event_flag(event: Event) -> bool {
            let sr = Self::block().sr.read();
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

        /// 使能或屏蔽事件
        #[inline]
        fn event_config(event: Event, en: bool) {
            Self::block().dier.modify(|_, w| match event {
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
            })
        }

        /// 事件清除
        #[inline]
        fn event_clear(event: Event) {
            Self::block().sr.modify(|_, w| match event {
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
            });
        }

        /// 根据需要定时的ticks数计算出分频、重复、计数寄存器的值
        fn micros_to_compute_with_rep(micros: u64) -> (u16, u8, u16) {
            let ticks = micros * Self::get_time_pclk() as u64 / 1000_000;

            let psc = ticks / (1u64 << 24);
            let count = ticks / (psc + 1);

            let rep = count / (1u64 << 16);
            let arr = count / (rep + 1);

            (psc as u16, rep as u8, arr as u16)
        }

        /// 根据给定的纳秒计算分频和重复、计数寄存器的值
        fn nanosecond_to_compute_with_rep(nano: u64) -> (u16, u8, u16) {
            let ticks = nano * Self::get_time_pclk() as u64 / 1000_000_000;

            let psc = ticks / (1u64 << 24);
            let count = ticks / (psc + 1);

            let rep = count / (1u64 << 16);
            let arr = count / (rep + 1);

            (psc as u16, rep as u8, arr as u16)
        }
    }
}

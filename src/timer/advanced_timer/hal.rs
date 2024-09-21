pub(crate) mod sealed {
    use super::super::Event;
    use super::super::*;
    use crate::clock::timer_pclk;
    use crate::pac;

    pub trait Instance {
        // 考虑以后其他单片机可能有多个IIC
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

        /// 单脉冲模式
        /// 0：在发生更新事件时，计数器不停止
        /// 1：在发生下一次更新事件(清除 CEN 位)时，计数器停止。
        #[inline]
        fn enable_single_mode(en: bool) {
            Self::block().cr1.modify(|_, w| w.opm().bit(en))
        }

        /// 使能自动重载
        #[inline]
        fn enable_auto_reload_buff(en: bool) {
            // 自动重装载预装载允许位
            // 0： TIM1_ARR 寄存器没有缓冲
            // 1： TIM1_ARR 寄存器被装入缓冲器
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

        /// 基本配置
        #[inline]
        fn base_config(config: BaseConfig) -> Result<(), Error> {
            // 设置计数对齐模式
            Self::set_cms(config.center_aligned_mode);
            // 设置计数方向
            Self::set_dir(config.count_direction);
            // 设置预分频
            Self::set_prescaler(config.prescaler);

            Ok(())
        }

        /// 返回更新事件的标志
        #[inline]
        fn update_flag() -> bool {
            Self::block().sr.read().uif().bit()
        }

        /// 清除更新事件标志
        #[inline]
        fn update_flag_clear() {
            Self::block().sr.modify(|_, w| w.uif().clear_bit())
        }

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

        #[inline]
        fn enable_event(event: Event, en: bool) {
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

        fn micros_to_compute_with_rep(micros: u64) -> (u16, u8, u16) {
            let ticks = micros * Self::get_time_pclk() as u64 / 1000_000;

            let psc = ticks / (1u64 << 24);
            let div = psc;
            let count = ticks / (div + 1);

            let rep = count / (1u64 << 16);
            let arr = count / (psc + 1);
            (div as u16, rep as u8, arr as u16)
        }
    }
}

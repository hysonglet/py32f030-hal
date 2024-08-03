pub(crate) mod sealed {

    use super::super::*;
    use crate::clock::timer_pclk;
    // use crate::common::Peripheral;
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
        fn counter_frequency() -> u32 {
            let psc: u32 = Self::block().psc.read().bits();
            // 计数器的时钟频率（CK_CNT）等于 fCK_PSC/( PSC[15:0]+1)。
            timer_pclk() / (psc + 1)
        }

        fn set_repetition(repetition: u16) {
            Self::block()
                .rcr
                .write(|w| unsafe { w.bits(repetition as u32) })
        }

        /// 基本配置
        fn base_config(config: BaseConfig) -> Result<(), Error> {
            // 设置计数对齐模式
            Self::set_cms(config.center_aligned_mode);
            // 设置计数方向
            Self::set_dir(config.count_direction);
            // 设置预分频
            Self::set_prescaler(config.prescaler);

            // 设置计数值
            // Self::set_cnt(config.period);
            // 设置周期值
            // Self::set_period_cnt(config.period);

            // // 设置重载模式
            // Self::set_auto_reload(config.auto_reload);
            // Self::enable_auto_reload_buff(true);

            // 默认值设置为重载值
            // Self::set_cnt(0);

            // 设置重复模式

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

        /// 产生更新事件。该位由软件置 1，硬件自动清 0。
        /// 0：无动作； 1：重新初始化计数器，并产生一个更新事件。注意：预分频器的计数器也被清 0(但是预分频系数不变)。若在中心对称模式下或 DIR=0(向上计数)则计数器被清 0，若 DIR=1(向下计数)则计数器装载 TIM1_ARR的值。
        fn triggle_update() {
            unsafe { Self::block().egr.write_with_zero(|w| w.ug().set_bit()) }
        }
    }
}

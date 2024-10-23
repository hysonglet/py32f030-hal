pub mod sealed {
    use super::super::*;
    use crate::bit::*;
    use crate::pac;

    pub(crate) trait Instance {
        fn id() -> Id;

        #[inline]
        fn block() -> &'static pac::adc::RegisterBlock {
            match Self::id() {
                Id::ADC1 => unsafe { pac::ADC::PTR.as_ref().unwrap() },
            }
        }

        /// 开始校准
        #[inline]
        fn calibration_start() {
            Self::block().cr.modify(|_, w| w.adcal().set_bit());
        }

        /// 使能 adc 外设
        #[inline]
        fn enable() {
            Self::block().cr.modify(|_, w| w.aden().set_bit())
        }

        /// 关闭 adc 使能
        #[inline]
        fn disable() {
            Self::block().cr.modify(|_, w| w.aden().clear_bit())
        }

        /// 设置时钟模式
        #[inline]
        fn set_clock_mode(clock: ClockMode) {
            Self::block()
                .cfgr2
                .modify(|_, w| unsafe { w.ckmode().bits(clock as u8) })
        }

        /// 启动转换
        #[inline]
        fn start() {
            Self::block().cr.modify(|_, w| w.adstart().set_bit());
        }

        /// 停止转换
        #[inline]
        fn stop() {
            Self::block().cr.modify(|_, w| w.adstp().set_bit());
        }

        // #[inline]
        // fn analog_watch_dog_select(channel: AdcChannel) {
        //     Self::block()
        //         .cfgr1
        //         .modify(|_, w| unsafe { w.awdch().bits(channel as u8) });
        // }

        // #[inline]
        // fn analog_watch_dog_enable(en: bool) {
        //     Self::block().cfgr1.modify(|_, w| w.awden().bit(en));
        // }

        #[inline]
        fn conversion_mode(mode: ConversionMode) {
            let (cont, discen) = match mode {
                ConversionMode::Single => (false, false),
                ConversionMode::Continuous => (true, false),
                ConversionMode::Discontinuous => (false, true),
            };
            Self::block()
                .cfgr1
                .modify(|_, w| w.discen().bit(discen).cont().bit(cont))
        }

        /// 设置过写
        #[inline]
        fn set_overwrite(en: bool) {
            Self::block().cfgr1.modify(|_, w| w.ovrmod().bit(en));
        }

        // /// 设置等待模式
        // fn wait_mode(en: bool) {
        //     Self::block().cfgr1.modify(|_, w| w.wait().bit(en));
        // }

        /// 触发信号的类型
        #[inline]
        fn trigle_signal(trigle: TrigleSignal) {
            let (exten, extsel) = match trigle {
                TrigleSignal::Soft => (0, 0),
                TrigleSignal::Rising(s) => (1, s as u8),
                TrigleSignal::Falling(s) => (2, s as u8),
                TrigleSignal::RisingFalling(s) => (3, s as u8),
            };
            Self::block()
                .cfgr1
                .modify(|_, w| unsafe { w.exten().bits(exten).extsel().bits(extsel) });
        }

        /// 配置为软件触发
        #[inline]
        fn is_soft_trigle() -> bool {
            Self::block().cfgr1.read().exten().bits() == 0
        }

        /// 设置对齐格式
        #[inline]
        fn align(align: Align) {
            Self::block()
                .cfgr1
                .modify(|_, w| w.align().bit(Align::Left == align))
        }

        /// 设置 adc 的精度
        #[inline]
        fn set_resolution(bit: Resolution) {
            Self::block()
                .cfgr1
                .modify(|_, w| unsafe { w.ressel().bits(bit as u8) })
        }

        #[inline]
        fn set_scan_dir(dir: ScanDir) {
            Self::block()
                .cfgr1
                .modify(|_, w| w.scandir().bit(dir == ScanDir::Down))
        }

        // /// 设置dma模式
        // #[inline]
        // fn set_dma_mode(mode: DmaMode) {
        //     Self::block()
        //         .cfgr1
        //         .modify(|_, w| w.dmacfg().bit(mode == DmaMode::Cycle))
        // }

        // /// dma 使能
        // #[inline]
        // fn dma_enable(en: bool) {
        //     Self::block().cfgr1.modify(|_, w| w.dmaen().bit(en))
        // }

        /// 设置转换的采样周期
        #[inline]
        fn set_sample_cycle(cycle: SampleCycles) {
            Self::block()
                .smpr
                .modify(|_, w| unsafe { w.smp().bits(cycle as u8) })
        }

        /// 设置看门狗预支
        #[inline]
        fn set_watch_dog_threshold(high: u16, low: u16) {
            Self::block()
                .tr
                .modify(|_, w| unsafe { w.ht().bits(high).lt().bits(low) })
        }

        /// 通道使能或关闭
        #[inline]
        fn channel_enable(channel: AdcChannel, en: bool) {
            // 仅当 ADSART=0 时（确保没有正在进行的转换）允许软件写该位
            Self::block().chselr.modify(|r, w| unsafe {
                w.bits(bit_mask_idx_modify::<1>(
                    channel as usize,
                    r.bits(),
                    if en { 1 } else { 0 },
                ))
            });
            if channel == AdcChannel::Channel11 {
                Self::block().ccr.modify(|_, w| w.tsen().bit(en))
            } else if channel == AdcChannel::Channel12 {
                Self::block().ccr.modify(|_, w| w.vrefen().bit(en))
            }
        }

        #[allow(dead_code)]
        fn channel_enable_exclusive(channel: AdcChannel) {
            // 仅当 ADSART=0 时（确保没有正在进行的转换）允许软件写该位
            Self::block()
                .chselr
                .write(|w| unsafe { w.bits(bit_mask_idx::<1>(channel as usize)) });
            if channel == AdcChannel::Channel11 {
                Self::block().ccr.modify(|_, w| w.tsen().bit(true))
            } else if channel == AdcChannel::Channel12 {
                Self::block().ccr.modify(|_, w| w.vrefen().bit(true))
            }
        }

        /// 返回转换的数据寄存器内容
        #[inline]
        fn data_read() -> u16 {
            Self::block().dr.read().data().bits()
        }

        /// 设置校准采样时间
        #[inline]
        fn set_calibration_sample_time(time: CalibrationSampleTime) {
            Self::block()
                .ccsr
                .modify(|_, w| unsafe { w.calsmp().bits(time as u8) })
        }

        /// 校准内容
        #[inline]
        fn set_calibration_content(select: CalibrationSelect) {
            Self::block()
                .ccsr
                .modify(|_, w| w.calsel().bit(select == CalibrationSelect::OffsetLinearity))
        }

        #[inline]
        fn event_flag(event: Event) -> bool {
            let isr = Self::block().isr.read();
            match event {
                Event::EOSMP => isr.eosmp().bit(),
                Event::EOC => isr.eoc().bit(),
                Event::EOSEQ => isr.eoseq().bit(),
                Event::OVR => isr.ovr().bit(),
                Event::AWD => isr.awd().bit(),
            }
        }

        #[inline]
        #[allow(dead_code)]
        fn event_config(event: Event, en: bool) {
            Self::block().ier.modify(|_, w| match event {
                Event::EOSMP => w.eosmpie().bit(en),
                Event::EOC => w.eocie().bit(en),
                Event::EOSEQ => w.eoseqie().bit(en),
                Event::OVR => w.ovrie().bit(en),
                Event::AWD => w.awdie().bit(en),
            });
        }

        #[inline]
        #[allow(dead_code)]
        fn event_clear(event: Event) {
            Self::block().isr.modify(|_, w| match event {
                Event::EOSMP => w.eosmp().set_bit(),
                Event::EOC => w.eoc().set_bit(),
                Event::EOSEQ => w.eoseq().set_bit(),
                Event::OVR => w.ovr().set_bit(),
                Event::AWD => w.awd().set_bit(),
            });
        }
    }
}

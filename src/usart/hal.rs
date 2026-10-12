pub mod sealed {
    use super::super::*;
    use crate::clock;
    use crate::pac;

    pub trait Instance {
        fn id() -> Id;

        /// USART1 / USART2 共用 `usart1` 寄存器块布局，仅基址不同
        #[inline]
        fn block() -> &'static pac::usart1::RegisterBlock {
            match Self::id() {
                Id::USART1 => unsafe { pac::USART1::PTR.as_ref().unwrap() },
                Id::USART2 => unsafe { pac::USART2::PTR.as_ref().unwrap() },
            }
        }

        ////////////////// 时钟 / 使能与停止 //////////////////

        #[inline]
        fn enable() {
            Self::id().clock().open()
        }

        #[inline]
        fn disable() {
            Self::id().clock().close()
        }

        #[inline]
        fn reset() {
            Self::id().clock().reset()
        }

        #[inline]
        fn start() {
            Self::block().cr1.modify(|_, w| w.ue().bit(true))
        }

        #[inline]
        fn stop() {
            Self::block().cr1.modify(|_, w| w.ue().bit(false))
        }

        ////////////////// 基本收发 //////////////////

        #[inline]
        fn read() -> u8 {
            Self::block().dr.read().dr().bits() as u8
        }

        /// 读取 9 位数据（DR 为 9 位宽，用于 9 位数据位模式）
        #[inline]
        fn read_u16() -> u16 {
            Self::block().dr.read().dr().bits()
        }

        #[inline]
        fn write(data: u8) {
            Self::block()
                .dr
                .write(|w| unsafe { w.dr().bits(data as u16) });
        }

        /// 写入 9 位数据（低 9 位有效）
        #[inline]
        fn write_u16(data: u16) {
            Self::block()
                .dr
                .write(|w| unsafe { w.dr().bits(data & 0x01ff) });
        }

        #[inline]
        fn rx_ready() -> bool {
            Self::event_flag(Event::RXNE)
        }

        #[inline]
        fn tx_ready() -> bool {
            Self::event_flag(Event::TXE)
        }

        #[inline]
        fn read_byte_blocking() -> u8 {
            while !Self::event_flag(Event::RXNE) {}
            Self::read()
        }

        #[inline]
        fn read_bytes_blocking(buf: &mut [u8]) -> usize {
            let len = buf.len();
            for item in buf {
                *item = Self::read_byte_blocking()
            }
            len
        }

        fn read_bytes_idle_blocking(buf: &mut [u8]) -> usize {
            let mut cnt = 0;
            for item in buf {
                loop {
                    if Self::event_flag(Event::RXNE) {
                        break;
                    }
                    // 线路空闲：本次传输结束
                    if Self::event_flag(Event::IDLE) {
                        // 读 SR 再读 DR 以清除 IDLE，读到的字节无效
                        Self::event_clear(Event::IDLE);
                        return cnt;
                    }
                }
                cnt += 1;
                *item = Self::read();
            }
            cnt
        }

        #[inline]
        fn write_byte_blocking(data: u8) -> Result<(), Error> {
            while !Self::event_flag(Event::TXE) {}
            Self::write(data);
            Ok(())
        }

        #[inline]
        fn write_bytes_blocking(buf: &[u8]) -> Result<(), Error> {
            for item in buf {
                Self::write_byte_blocking(*item)?;
            }
            Ok(())
        }

        #[inline]
        fn tx_flush() {
            while !Self::event_flag(Event::TC) {}
        }

        /// 发送断开符（置位 CR1.SBK）
        #[inline]
        fn send_break() {
            Self::block().cr1.modify(|_, w| w.sbk().set_bit());
        }

        /// 停止发送断开符
        #[inline]
        fn clear_break() {
            Self::block().cr1.modify(|_, w| w.sbk().clear_bit());
        }

        ////////////////// 收发使能 //////////////////

        #[inline]
        fn rx_enable(en: bool) {
            Self::block().cr1.modify(|_, w| w.re().bit(en))
        }

        #[inline]
        fn tx_enable(en: bool) {
            Self::block().cr1.modify(|_, w| w.te().bit(en))
        }

        #[inline]
        fn rts_enable(en: bool) {
            Self::block().cr3.modify(|_, w| w.rtse().bit(en))
        }

        #[inline]
        fn cts_enable(en: bool) {
            Self::block().cr3.modify(|_, w| w.ctse().bit(en))
        }

        /// 同时设置 RTS / CTS（CR3.RTSE / CR3.CTSE）
        #[inline]
        fn set_flow_control(rts: bool, cts: bool) {
            Self::block()
                .cr3
                .modify(|_, w| w.ctse().bit(cts).rtse().bit(rts))
        }

        ////////////////// DMA //////////////////

        #[inline]
        fn rx_dma_enable(en: bool) {
            Self::block().cr3.modify(|_, w| w.dmar().bit(en));
        }

        #[inline]
        fn tx_dma_enable(en: bool) {
            Self::block().cr3.modify(|_, w| w.dmat().bit(en));
        }

        ////////////////// 半双工 / IrDA / 同步时钟 / 静默 //////////////////

        /// 半双工（单线）模式：CR3.HDSEL
        #[inline]
        fn set_half_duplex(en: bool) {
            Self::block().cr3.modify(|_, w| w.hdsel().bit(en));
        }

        /// IrDA 模式：`Some(mode)` 使能 CR3.IREN 并设置低功耗位 IRLP
        #[inline]
        fn set_irda(mode: Option<IrdaMode>) {
            let en = mode.is_some();
            let low_power = matches!(mode, Some(IrdaMode::LowPower));
            Self::block()
                .cr3
                .modify(|_, w| w.iren().bit(en).irlp().bit(low_power));
        }

        /// 同步模式时钟输出（CR2.CLKEN / CPOL / CPHA / LBCL）
        #[inline]
        fn set_sync_clock(cfg: Option<SyncClockConfig>) {
            Self::block().cr2.modify(|_, w| {
                let w = w.clken().bit(cfg.is_some());
                match cfg {
                    Some(c) => w
                        .cpol()
                        .bit(c.polarity == ClockPolarity::IdleHigh)
                        .cpha()
                        .bit(c.phase == ClockPhase::SecondEdge)
                        .lbcl()
                        .bit(c.last_bit_clock),
                    None => w
                        .cpol()
                        .bit(false)
                        .cpha()
                        .bit(false)
                        .lbcl()
                        .bit(false),
                }
            });
        }

        /// 静默模式（CR1.RWU）与唤醒方式（CR1.WAKE）
        #[inline]
        fn set_mute(mode: Option<WakeMethod>) {
            Self::block().cr1.modify(|_, w| {
                w.rwu()
                    .bit(mode.is_some())
                    .wake()
                    .bit(matches!(mode, Some(WakeMethod::AddressMark)))
            });
        }

        /// 多处理器通信节点地址（CR2.ADD）
        #[inline]
        fn set_address(address: u8) {
            Self::block()
                .cr2
                .modify(|_, w| unsafe { w.add().bits(address) });
        }

        ////////////////// 自动波特率 //////////////////

        /// 使能自动波特率检测并设置测量模式（CR3.ABREN / ABRMOD）
        #[inline]
        fn set_auto_baud(mode: Option<AutoBaudMode>) {
            Self::block().cr3.modify(|_, w| {
                w.abren().bit(mode.is_some());
                unsafe { w.abrmod().bits(mode.unwrap_or_default() as u8) }
            });
        }

        /// 请求重新进行自动波特率测量（写 1 到 SR.ABRRQ）
        #[inline]
        fn request_auto_baud() {
            Self::block().sr.modify(|_, w| w.abrrq().set_bit());
        }

        /// 读取自动波特率测量结果对应的 BRR 值
        #[inline]
        fn auto_baud_result() -> u16 {
            Self::block().brr.read().bits() as u16
        }

        ////////////////// 事件 //////////////////

        /// 清除事件标志
        fn event_clear(event: Event) {
            Self::block().sr.modify(|r, w| match event {
                // PE / FE / NE / ORE / IDLE：需先读 SR 再读 DR 才能清零
                Event::PE | Event::FE | Event::NE | Event::ORE | Event::IDLE => {
                    let _ = r.bits();
                    let _ = Self::read();
                    w
                }
                Event::RXNE => w.rxne().clear_bit(),
                Event::TC => w.tc().clear_bit(),
                // TXE 由写 DR 清零
                Event::TXE => w,
                Event::CTS => w.cts().clear_bit(),
                // ABRE：写 1 到 ABRRQ 请求重新测量
                Event::ABRE => w.abrrq().set_bit(),
                // ABRF 由硬件在下次测量时清零
                Event::ABRF => w,
            });
        }

        /// 返回事件标志
        fn event_flag(event: Event) -> bool {
            let sr = Self::block().sr.read();
            match event {
                Event::PE => sr.pe(),
                Event::FE => sr.fe(),
                Event::NE => sr.ne(),
                Event::ORE => sr.ore(),
                Event::IDLE => sr.idle(),
                Event::RXNE => sr.rxne(),
                Event::TC => sr.tc(),
                Event::TXE => sr.txe(),
                Event::CTS => sr.cts(),
                Event::ABRE => sr.abre(),
                Event::ABRF => sr.abrf(),
            }
            .bit()
        }

        /// 开启或关闭事件中断
        fn event_config(event: Event, en: bool) {
            let block = Self::block();
            match event {
                Event::PE => block.cr1.modify(|_, w| w.peie().bit(en)),
                // FE / NE / ORE 共用 CR3.EIE
                Event::FE | Event::NE | Event::ORE => {
                    block.cr3.modify(|_, w| w.eie().bit(en));
                }
                Event::IDLE => block.cr1.modify(|_, w| w.idleie().bit(en)),
                Event::RXNE => block.cr1.modify(|_, w| w.rxneie().bit(en)),
                Event::TC => block.cr1.modify(|_, w| w.tcie().bit(en)),
                Event::TXE => block.cr1.modify(|_, w| w.txeie().bit(en)),
                // CTS 中断：CR3.CTSIE
                Event::CTS => block.cr3.modify(|_, w| w.ctsie().bit(en)),
                // ABRE / ABRF 无独立中断使能位，随 ABR 过程由硬件置位
                Event::ABRE => {}
                Event::ABRF => {}
            }
        }

        /// 返回事件中断是否已使能
        fn is_event_enable(event: Event) -> bool {
            let block = Self::block();
            match event {
                Event::PE => block.cr1.read().peie().bit(),
                // 三者共用 CR3.EIE
                Event::FE | Event::NE | Event::ORE => block.cr3.read().eie().bit(),
                Event::IDLE => block.cr1.read().idleie().bit(),
                Event::RXNE => block.cr1.read().rxneie().bit(),
                Event::TC => block.cr1.read().tcie().bit(),
                Event::TXE => block.cr1.read().txeie().bit(),
                Event::CTS => block.cr3.read().ctsie().bit(),
                Event::ABRE => false,
                Event::ABRF => false,
            }
        }

        ////////////////// 波特率 //////////////////

        /// 整数方式计算 BRR，避免引入浮点运算
        ///
        /// 过采样 16 时：`USARTDIV = f_PCLK / (16 * baud)`
        /// 过采样 8  时：`USARTDIV = f_PCLK / (8 * baud)`（小数位仅 3 位有效，bit3 保持 0）
        fn set_baud_rate(baud_rate: u32, over_sampling: OverSampling) {
            let pclk = clock::sys_pclk() as u64;
            let mul: u64 = if over_sampling == OverSampling::Eight {
                2
            } else {
                1
            };

            // USARTDIV 以 1/16 为单位，并做四舍五入
            let div16 = (pclk * mul + baud_rate as u64 / 2) / baud_rate as u64;
            let mantissa = (div16 / 16) as u16;
            let mut fraction = (div16 % 16) as u8;
            if over_sampling == OverSampling::Eight {
                fraction &= 0x7;
            }

            Self::block().brr.modify(|_, w| unsafe {
                w.div_mantissa()
                    .bits(mantissa)
                    .div_fraction()
                    .bits(fraction)
            });
        }

        ////////////////// 配置 //////////////////

        /// 配置串口（必须在 UE=0 时写入，故先停止再启动）
        fn config(config: Config) {
            let block = Self::block();

            Self::stop();

            // 停止位
            block
                .cr2
                .modify(|_, w| unsafe { w.stop().bits(config.stop_bit as u8) });

            // 数据位 + 校验使能 + 校验极性（PS：0 偶校验，1 奇校验）
            block.cr1.modify(|_, w| {
                w.m()
                    .bit(config.data_bits.into())
                    .pce()
                    .bit(config.parity.is_enable())
                    .ps()
                    .bit(config.parity == Parity::Odd)
            });

            // 过采样
            block
                .cr3
                .modify(|_, w| w.over8().bit(config.over_sampling.into()));

            // 硬件流控
            Self::set_flow_control(config.hw_flow_ctrl.rts(), config.hw_flow_ctrl.cts());

            // 自动波特率
            Self::set_auto_baud(config.auto_baud);

            // 半双工
            Self::set_half_duplex(config.half_duplex);

            // IrDA
            Self::set_irda(config.irda);

            // 同步时钟输出
            Self::set_sync_clock(config.sync_clock);

            // 静默模式与唤醒方式 + 节点地址
            Self::set_mute(config.mute);
            Self::set_address(config.address);

            // 波特率
            Self::set_baud_rate(config.baud_rate.0, config.over_sampling);

            Self::start();
        }
    }
}

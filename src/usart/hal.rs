pub mod sealed {
    use super::super::*;
    use crate::pac;
    use core::f32;
    use core::intrinsics::floorf32;

    pub trait Instance {
        fn id() -> Id;

        #[inline]
        fn block() -> &'static pac::usart1::RegisterBlock {
            match Self::id() {
                Id::USART1 => unsafe { pac::USART1::PTR.as_ref().unwrap() },
                Id::USART2 => unsafe { pac::USART2::PTR.as_ref().unwrap() },
            }
        }

        #[inline]
        fn enable() {
            Self::id().open()
        }

        #[inline]
        fn reset() {
            Self::id().reset()
        }

        #[inline]
        fn start() {
            Self::block().cr1.modify(|_, w| w.ue().bit(true))
        }

        #[inline]
        fn stop() {
            Self::block().cr1.modify(|_, w| w.ue().bit(false))
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
                while !Self::event_flag(Event::RXNE) {
                    if Self::event_flag(Event::IDLE) {
                        *item = Self::read();
                        return cnt;
                    }
                }
                cnt += 1;
                *item = Self::read()
            }
            cnt
        }

        #[inline]
        fn write(data: u8) {
            Self::block()
                .dr
                .write(|w| unsafe { w.dr().bits(data.into()) });
        }

        #[inline]
        fn read() -> u8 {
            Self::block().dr.read().dr().bits() as u8
        }

        #[inline]
        fn write_byte_blocking(data: u8) {
            // txe: 0: 未传输完， 1： 传输完毕
            while !Self::event_flag(Event::TXE) {}
            Self::write(data);
        }

        #[inline]
        fn write_bytes_blocking(buf: &[u8]) -> usize {
            let len = buf.len();
            for item in buf {
                Self::write_byte_blocking(*item);
            }
            len
        }

        #[inline]
        fn set_flow_control(rts: bool, cts: bool) {
            let block = Self::block();
            block.cr3.modify(|_, w| w.ctse().bit(cts).rtse().bit(rts))
        }

        #[inline]
        fn rx_enable(en: bool) {
            Self::block().cr1.modify(|_, w| w.re().bit(en))
        }

        #[inline]
        fn rx_ready() -> bool {
            Self::block().sr.read().rxne().bit()
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

        /// 清除事件标志
        fn event_clear(event: Event) {
            Self::block().sr.modify(|r, w| match event {
                Event::PE | Event::FE | Event::NE | Event::ORE | Event::IDLE => {
                    // ﻿当接收时校验值错误时，硬件置位该寄存器。
                    // 软件先读 USART_SR 寄存器后读 USART_DR 寄存器可以清零该位。但软件在清该位前必须等待RXNE=1
                    let _ = r.bits();
                    w
                }
                Event::RXNE => {
                    // ﻿软件读 USART_DR 寄存器、或者写 0 清零该位
                    w.rxne().clear_bit()
                }
                Event::TC => {
                    // ﻿软件先读 USART_SR 寄存器然后写 USART_DR寄存器会清零该位（针对多处理器通讯）。软件同时可以写 0 清零。
                    w.tc().clear_bit()
                }
                Event::TXE => {
                    // ﻿当 USART_DR 寄存器数据传送到移位寄存器，硬件置位该寄存器。当 TXEIE=1 时，产生中断。写 USART_DR 寄存器会清零该位
                    w
                }
                Event::CTS => {
                    // ﻿当 CTS 输入 toggle，别 CTSE=1 时，该寄存器为 1.软件写 0 清零。当 CTSIE=1 时，产生 CTS中断
                    w.cts().clear_bit()
                }
                Event::ABRE => {
                    // ﻿当自动波特率检测出错（波特率超出范围或者字符比较错误）时，硬件置位该寄存器。软件通过写 1 到 ABRRQ 寄存器清零该位。
                    w.abrrq().set_bit()
                }
                Event::ABRF => {
                    // ﻿软件通过写 1 到 USART_RQR 寄存器的 ABRRQ位清零该位。
                    w
                }
            });
        }

        /// 返回
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
                Event::FE | Event::NE | Event::ORE => {
                    block.cr3.modify(|_, w| w.eie().bit(en));
                }
                Event::IDLE => block.cr1.modify(|_, w| w.idleie().bit(en)),
                Event::RXNE => block.cr1.modify(|_, w| w.rxneie().bit(en)),
                Event::TC => block.cr1.modify(|_, w| w.tcie().bit(en)),
                Event::TXE => block.cr1.modify(|_, w| w.txeie().bit(en)),
                Event::CTS => {}
                Event::ABRE => {}
                Event::ABRF => {}
            }
        }

        /// return event config
        fn is_event_enable(event: Event) -> bool {
            let cr1 = Self::block().cr1.read();
            match event {
                Event::PE => cr1.peie().bit(),
                Event::FE => false,
                Event::NE => false,
                Event::ORE => false,
                Event::IDLE => cr1.idleie().bit(),
                Event::RXNE => cr1.rxneie().bit(),
                Event::TC => cr1.tcie().bit(),
                Event::TXE => cr1.txeie().bit(),
                Event::CTS => false,
                Event::ABRE => false,
                Event::ABRF => false,
            }
        }

        /// 配置串口
        fn config(config: Config) {
            let block = Self::block();

            // 必须在串口停止状态下才能重新配置
            Self::stop();

            // 设置停止位
            block
                .cr2
                .modify(|_, w| unsafe { w.stop().bits(config.stop_bit as u8) });

            // 设置数据位数
            block.cr1.modify(|_, w| {
                w.m()
                    .bit(config.data_bits.into())
                    .pce()
                    .bit(config.parity != Parity::None)
            });

            // 设置奇偶校验
            block
                .cr1
                .modify(|_, w| w.pce().bit(config.parity != Parity::None));

            // 奇校验或偶校验，设置检验
            if config.parity != Parity::None {
                block
                    .cr1
                    .modify(|_, w| w.ps().bit(config.parity == Parity::Odd));
            }

            // 使能发送和接收
            // block.cr1.modify(|_, w| w.te().set_bit().re().set_bit());

            // 设置过采样
            block
                .cr3
                .modify(|_, w| w.over8().bit(config.over_sampling.into()));

            // 设置波特率
            let over_sampling: u32 = config.over_sampling.div();
            let div: f32 =
                clock::sys_pclk() as f32 / (config.baud_rate as u32 * over_sampling) as f32;
            let mantissa: u16 = unsafe { floorf32(div) } as u16;
            let fraction: u8 = (16.0 * (div - mantissa as f32)) as u8;
            block.brr.modify(|_, w| unsafe {
                w.div_mantissa()
                    .bits(mantissa)
                    .div_fraction()
                    .bits(fraction)
            });

            // 开启串口
            Self::start();
        }
    }
}

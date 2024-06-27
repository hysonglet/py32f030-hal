use core::f32;
use core::intrinsics::floorf32;

use super::*;

pub mod sealed {
    use super::*;
    use crate::pac;

    pub trait Instance {
        fn id() -> Usart;
        #[inline]
        fn block() -> &'static pac::usart1::RegisterBlock {
            match Self::id() {
                Usart::USART1 => unsafe { pac::USART1::PTR.as_ref().unwrap() },
                Usart::USART2 => unsafe { pac::USART2::PTR.as_ref().unwrap() },
            }
        }

        #[inline]
        fn enable(en: bool) {
            Self::id().enable(en)
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
        fn read_byte_block() -> u8 {
            let block = Self::block();

            while block.sr.read().rxne().bit_is_clear() {}

            block.dr.read().bits() as u8
        }

        #[inline]
        fn read_bytes_block(buf: &mut [u8]) {
            for item in buf {
                *item = Self::read_byte_block()
            }
        }

        #[inline]
        fn write_byte_block(data: u8) {
            let block = Self::block();

            // txe: 0: 未传输完， 1： 传输完毕
            while block.sr.read().txe().bit_is_clear() {}
            block.dr.write(|w| unsafe { w.bits(data as u32) })
        }

        #[inline]
        fn write_bytes_block(buf: &[u8]) {
            for item in buf {
                Self::write_byte_block(*item);
            }
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

        fn config(config: Config) {
            let block = Self::block();

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

            Self::start();
        }
    }
}

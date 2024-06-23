use core::f32;
use core::intrinsics::floorf32;

use crate::clock::{self, PllSelect};
use crate::common::Peripheral;
use crate::pac;

struct Usart1;
struct Usart2;

#[derive(Debug)]
pub enum Error {
    StartTimeout,
    ReadTimeout,
    WriteTimeout,
}

#[derive(Default)]
pub enum StopBits {
    #[default]
    Stop1 = 0,
    Stop2 = 1,
}

#[derive(Default)]
pub enum WordLen {
    #[default]
    WordLen8 = 0,
    WordLen9 = 1,
}

#[derive(Default, PartialEq)]
pub enum Parity {
    #[default]
    None = 0,
    Even = 1,
    Odd = 2,
}

#[derive(Default)]
pub enum HwFlowCtrl {
    #[default]
    None,
    Rts = 1,
    Cts = 2,
    RtsCts = 3,
}

#[derive(Default)]
pub enum BaudRate {
    // Auto = 0,
    Bps300 = 300,
    Bps1200 = 1200,
    Bps2400 = 2400,
    Bps4800 = 4800,
    Bps9600 = 9600,
    Bps1440 = 1440,
    Bps19200 = 19200,
    Bps28800 = 28800,
    Bps38400 = 38400,
    Bps57600 = 57600,
    Bps74880 = 74880,
    #[default]
    Bps115200 = 115200,
    Bps230400 = 230400,
}

#[derive(Default, PartialEq)]
pub enum OverSampling {
    #[default]
    OverSampling16 = 0,
    OverSampling8 = 1,
}

impl From<OverSampling> for bool {
    fn from(value: OverSampling) -> Self {
        value == OverSampling::OverSampling8
    }
}

#[derive(Default, PartialEq)]
pub enum DataBits {
    #[default]
    DataBits8 = 0,
    DataBits9 = 1,
}

impl From<DataBits> for bool {
    fn from(value: DataBits) -> Self {
        value == DataBits::DataBits9
    }
}

#[derive(Default)]
pub struct Config<T: UsartConfig> {
    pub baud_rate: BaudRate,
    pub stop_bit: StopBits,
    pub word_len: WordLen,
    pub parity: Parity,
    pub hw_flow_ctrl: HwFlowCtrl,
    pub data_bits: DataBits,

    pub over_sampling: OverSampling,

    pub mode: T,
}

pub trait UsartConfig {}

pub struct Usart {}
pub struct Uart {}

impl Peripheral for Usart1 {
    type Target = &'static pac::usart1::RegisterBlock;

    fn peripheral() -> Self::Target {
        unsafe { pac::USART1::ptr().as_ref().unwrap() }
    }
}

trait UsartHal {
    fn enable(en: bool);
    fn start() -> Result<(), Error>;
    fn stop() -> Result<(), Error>;
    fn config<T: UsartConfig>(config: Config<T>) -> Result<(), Error>;
    fn read_byte(timeout: usize) -> Result<u8, Error>;
    fn write_byte(data: u8, timeout: usize) -> Result<(), Error>;
    #[inline]
    fn read_block(buf: &mut [u8], timeout: usize) -> Result<usize, (usize, Error)> {
        let mut cnt = 0;
        for item in buf.iter_mut() {
            match Self::read_byte(timeout) {
                Ok(v) => {
                    *item = v;
                    cnt += 1;
                }
                Err(error) => {
                    return Err((cnt, error));
                }
            };
        }

        Ok(cnt)
    }
    #[inline]
    fn write_block(buf: &[u8], timeout: usize) -> Result<usize, (usize, Error)> {
        let mut cnt = 0;
        for item in buf.iter() {
            match Self::write_byte(*item, timeout) {
                Ok(_) => cnt += 1,
                Err(error) => {
                    return Err((cnt, error));
                }
            }
        }
        Ok(cnt)
    }
}

impl UsartHal for Usart1 {
    fn enable(en: bool) {
        clock::peripheral::PeripheralClock::USART1.enable(en)
    }

    fn stop() -> Result<(), Error> {
        Self::peripheral().cr1.modify(|_, w| w.ue().bit(false));
        Ok(())
    }

    fn start() -> Result<(), Error> {
        Self::peripheral().cr1.modify(|_, w| w.ue().bit(true));
        Ok(())
    }

    #[inline]
    fn read_byte(timeout: usize) -> Result<u8, Error> {
        let peripheral = Self::peripheral();
        let mut timout = timeout;
        while peripheral.sr.read().rxne().bit_is_clear() {
            cortex_m::asm::delay(1);
            if timout == 0 {
                return Err(Error::ReadTimeout);
            }
            timout -= 1;
        }

        Ok(peripheral.dr.read().bits() as u8)
    }

    #[inline]
    fn write_byte(data: u8, timeout: usize) -> Result<(), Error> {
        let peripheral = Self::peripheral();
        let mut timout = timeout;
        while peripheral.sr.read().txe().bit_is_clear() {
            cortex_m::asm::delay(1);
            if timout == 0 {
                return Err(Error::WriteTimeout);
            }
            timout -= 1;
        }

        peripheral.dr.write(|w| unsafe { w.bits(data as u32) });

        Ok(())
    }

    fn config<T: UsartConfig>(config: Config<T>) -> Result<(), Error> {
        let peripheral = Self::peripheral();

        Self::stop()?;

        // 设置停止位
        peripheral
            .cr2
            .modify(|_, w| unsafe { w.stop().bits(config.stop_bit as u8) });

        // 设置数据位数
        peripheral.cr1.modify(|_, w| {
            w.m()
                .bit(config.data_bits.into())
                .pce()
                .bit(config.parity != Parity::None)
        });

        // 设置奇偶校验
        peripheral
            .cr1
            .modify(|_, w| w.pce().bit(config.parity != Parity::None));
        if config.parity != Parity::None {
            peripheral
                .cr1
                .modify(|_, w| w.pce().bit(config.parity == Parity::Even));
        }

        // 使能发送和接收
        peripheral
            .cr1
            .modify(|_, w| w.te().set_bit().re().set_bit());

        // 设置过采样
        peripheral
            .cr3
            .modify(|_, w| w.over8().bit(config.over_sampling.into()));

        // 设置波特率
        let div: f32 = clock::sys_pclk() as f32 / config.baud_rate as u32 as f32;
        let mantissa: u16 = unsafe { floorf32(div) } as u16;
        let fraction: u8 = (16.0 * (div - mantissa as f32)) as u8;
        peripheral.brr.modify(|_, w| unsafe {
            w.div_mantissa()
                .bits(mantissa)
                .div_fraction()
                .bits(fraction)
        });

        Ok(())
    }
}

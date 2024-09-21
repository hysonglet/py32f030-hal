use super::{Error, Instance};
use crate::mode::{Blocking, Mode};
use core::{marker::PhantomData, u16};

/// 计数器
///
/// 使用向上计数模式
pub struct Counter<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _m: PhantomData<M>,
}

impl<'d, T: Instance, M: Mode> Counter<'d, T, M> {
    pub(super) fn new() -> Self {
        // 使用向上计数模式方便计数
        T::set_dir(super::CountDirection::Up);
        // 使用单脉冲模式
        T::enable_single_mode(true);
        // 不使用arr预装
        T::enable_auto_reload_buff(false);
        // 设置定时器分频频率为1M
        // assert!(
        //     T::get_time_pclk() >= freq,
        //     "timer pclk({}) must >= {}",
        //     T::get_time_pclk(),
        //     freq
        // );
        // let prescaler = (T::get_time_pclk() / freq - 1) as u32;
        // assert!(prescaler <= u16::MAX as u32, "freq too low");
        // T::set_prescaler(prescaler as u16);

        Counter {
            _t: PhantomData,
            _m: PhantomData,
        }
    }

    pub fn get_freq(&self) -> u32 {
        T::get_time_pclk() / (T::block().psc.read().bits() + 1)
    }
}

impl<'d, T: Instance> Counter<'d, T, Blocking> {
    pub fn start_block(&mut self, us: u32) {
        T::stop();
        T::update_flag_clear();
        T::set_cnt(0);

        if us <= u16::MAX as u32 {
            T::set_auto_reload(us as u16);
            T::set_repetition(0);
            T::enable_single_mode(true);
            T::start();
            defmt::info!("{}", us);
            while T::update_flag() == false {}
        } else {
            let repeatition = us / u16::MAX as u32;
            let remain = us % u16::MAX as u32;
            defmt::info!("{} {}", repeatition, remain);
            T::set_auto_reload(u16::MAX);
            T::set_repetition(repeatition as u16);
            T::enable_single_mode(false);
            T::start();
            while T::update_flag() == false {}
            T::set_auto_reload(remain as u16);
            while T::update_flag() == false {}
        }
    }

    /// 阻塞等待直到更新事件发生
    #[inline]
    pub fn delay_us_blocking(&mut self, us: u32) {
        T::stop();
        T::update_flag_clear();
        T::set_cnt(0);

        if us <= u16::MAX as u32 {
            T::set_auto_reload(us as u16);
            T::set_repetition(0);
            T::enable_single_mode(true);
            T::start();
            defmt::info!("{}", us);
            while T::update_flag() == false {}
        } else {
            let repeatition = us / u16::MAX as u32;
            let remain = us % u16::MAX as u32;
            defmt::info!("{} {}", repeatition, remain);
            T::set_auto_reload(u16::MAX);
            T::set_repetition(repeatition as u16);
            T::enable_single_mode(false);
            T::start();
            while T::update_flag() == false {}
            T::set_auto_reload(remain as u16);
            while T::update_flag() == false {}
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl<'d, T: Instance> embedded_hal::blocking::delay::DelayUs<u32> for Counter<'d, T, Blocking> {
    fn delay_us(&mut self, us: u32) {
        self.delay_us_blocking(us)
    }
}

impl<'d, T: Instance> embedded_hal::blocking::delay::DelayMs<u32> for Counter<'d, T, Blocking> {
    fn delay_ms(&mut self, ms: u32) {
        self.delay_us_blocking(ms * 1000);
    }
}

use fugit::{HertzU32, MicrosDurationU64, MillisDuration, MillisDurationU64, Rate};

impl<'d, T: Instance> embedded_hal::timer::CountDown for Counter<'d, T, Blocking> {
    type Time = MicrosDurationU64;
    fn start<H>(&mut self, count: H)
    where
        H: Into<Self::Time>,
    {
        defmt::info!("pck: {:x}", T::get_time_pclk());
        let micros = count.into().to_micros();

        // let period_micros: u64 = 1000_000 * 1000 / self.get_freq() as u64;
        // let ticks: u64 = micros / period_micros;
        // defmt::info!("ticks: {}, freq: {}", ticks, self.get_freq());
        // let micros = time.into().to_micros();
        let ticks = micros * freq as u64 /1000_000;

        let psc = ticks/(1u64<<32);
        let div = psc;
        let count = ticks/(div + 1);
        
        let psc = count/(1u64<<16);
        let arr = count/(psc + 1) -1 ;
        let (div, y, x) =  (div, psc, arr);
        
        let c: u64 = (div + 1)*(y + 1)*(x + 1);
        //println!("div: {} y: {} x: {} c: {} ticks: {} dif: {}", div, y, x, c, ticks, if c > ticks {c-ticks} else {ticks - c});


        let ticks = micros * T::get_time_pclk() as u64 / 1000_000;
        let psc = (ticks - 1) / (1 << 16);
        let arr = ticks / (psc + 1) - 1;

        let f: u32 = T::get_time_pclk() / (psc as u32 + 1);
        let p_us: f32 = 1000_000.0 / (T::get_time_pclk() as f32 / (psc as f32 + 1.0));
        let all = p_us * (arr as f32 + 1.0);
        defmt::info!(
            "pclk: {} micros: {} ticks: {} psc: {} arr: {} f: {} p_us: {} us {} diff: {}",
            T::get_time_pclk(),
            micros,
            ticks,
            psc,
            arr,
            f,
            p_us as u32,
            all as u32,
            micros - all as u64
        );

        // period = 1 * 1000_000 / T::get_time_pclk();
    }

    fn wait(&mut self) -> nb::Result<(), void::Void> {
        todo!()
    }
}

// impl<'d, T: Instance, const FREQ: u32> fugit_timer::Timer<FREQ> for Counter<'d, T, Blocking, FREQ> {
//     type Error = Error;

//     fn start(&mut self, duration: fugit::TimerDurationU32<FREQ>) -> Result<(), Self::Error> {
//         defmt::info!("{}", duration.ticks());
//         // todo!()
//         Ok(())
//     }

//     fn now(&mut self) -> fugit::TimerInstantU32<FREQ> {
//         todo!()
//     }

//     fn cancel(&mut self) -> Result<(), Self::Error> {
//         todo!()
//     }

//     fn wait(&mut self) -> nb::Result<(), Self::Error> {
//         todo!()
//     }
// }

use super::{Event, Instance};
use crate::{
    mode::{Blocking, Mode},
    timer::advanced_timer::{CenterAlignedMode, CountDirection},
};
use core::{marker::PhantomData, u16};
use fugit::MicrosDurationU64;

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
        // T::set_dir(super::CountDirection::Up);
        // 使用单脉冲模式
        // T::enable_single_mode(true);
        // 不使用arr预装
        // T::enable_auto_reload_buff(false);

        T::enable_auto_reload_buff(false);
        T::enable_single_mode(false);
        T::set_cms(CenterAlignedMode::EdgeAligned);
        T::set_dir(CountDirection::Down);

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
    /// 阻塞等待直到更新事件发生
    #[inline]
    pub fn delay_us_blocking(&mut self, us: u32) {
        let (div, rep, arr) = T::micros_to_compute_with_rep(us as u64);
        T::stop();
        T::set_prescaler(div);
        T::set_repetition(rep);
        T::set_auto_reload(arr);
        T::event_clear(Event::UIF);
        T::start();

        while T::event_flag(Event::UIF) == false {}
        T::stop();
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

impl<'d, T: Instance> embedded_hal::timer::CountDown for Counter<'d, T, Blocking> {
    type Time = MicrosDurationU64;
    fn start<H>(&mut self, count: H)
    where
        H: Into<Self::Time>,
    {
        let micros = count.into().to_micros();

        let (div, rep, arr) = T::micros_to_compute_with_rep(micros);

        // let (div, y, x) = (div as u16, psc, arr);
        // let c: u64 = (div as u64 + 1) * (y + 1) * (x + 1) as u64;
        // defmt::info!(
        //     "div: {} y: {} x: {} c: {} ticks: {} dif: {}",
        //     div,
        //     y,
        //     x,
        //     c,
        //     ticks,
        //     if c > ticks { c - ticks } else { ticks - c }
        // );

        T::stop();

        T::set_prescaler(div);
        T::set_repetition(rep);
        T::set_auto_reload(arr);
        T::event_clear(Event::UIF);

        T::start();
    }

    fn wait(&mut self) -> nb::Result<(), void::Void> {
        while T::event_flag(Event::UIF) == false {}
        T::stop();
        Ok(())
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

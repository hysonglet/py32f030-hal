use super::Instance;
use crate::mode::{Blocking, Mode};
use core::{marker::PhantomData, u16};

/// 计数器
///
/// 使用向上计数模式
pub struct Counter<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _m: PhantomData<M>,
}

// pub struct Config {
//     auto_reload: u16,
//     repetition: Option<u16>,
// }

// impl Config {
//     fn new(auto_reload: u16, repetition: Option<u16>) -> Self {
//         Self {
//             auto_reload,
//             repetition,
//         }
//     }
// }

impl<'d, T: Instance, M: Mode> Counter<'d, T, M> {
    pub(super) fn new() -> Self {
        // 使用向上计数模式方便计数
        T::set_dir(super::CountDirection::Up);
        // 使用单脉冲模式
        T::enable_single_mode(true);
        // 不使用arr预装
        T::enable_auto_reload_buff(false);
        // 设置定时器分频频率为1M
        assert!(T::get_time_pclk() >= 1_000_000, "timer pclk must >= 1M");
        let prescaler = (T::get_time_pclk() / 1_000_000 - 1) as u16;
        T::set_prescaler(prescaler);

        Counter {
            _t: PhantomData,
            _m: PhantomData,
        }
    }
}

impl<'d, T: Instance> Counter<'d, T, Blocking> {
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

use fugit::HertzU32;

impl<'d, T: Instance> embedded_hal::timer::CountDown for Counter<'d, T, Blocking> {
    type Time = HertzU32;
    fn start<H>(&mut self, count: H)
    where
        H: Into<Self::Time>,
    {
        todo!()
    }

    fn wait(&mut self) -> nb::Result<(), void::Void> {
        todo!()
    }
}

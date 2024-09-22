use super::{Channel, Event, Instance};
use crate::gpio::{self, AnyPin};
use crate::{
    mode::{Blocking, Mode},
    timer::advanced_timer::{CenterAlignedMode, CountDirection},
};
use core::marker::PhantomData;
use embassy_hal_internal::PeripheralRef;
use fugit::MicrosDurationU32;

/// 计数器
///
/// 使用向上计数模式
pub struct Pwm<'d, T: Instance, M: Mode> {
    _t: PhantomData<&'d T>,
    _m: PhantomData<M>,
}

pub struct PwmChannel<'d, T: Instance> {
    _id: Channel,
    _t: PhantomData<&'d T>,

    _ch_pin: Option<PeripheralRef<'d, AnyPin>>,
    _ch_n_pin: Option<PeripheralRef<'d, AnyPin>>,
}

impl<'d, T: Instance, M: Mode> Pwm<'d, T, M> {
    pub fn enable(&mut self, channel: Channel) {}

    pub fn disable(&mut self, channel: Channel) {}

    pub fn get_duty(&self, channel: Channel) {}

    pub fn get_max_duty(&self) -> u16 {
        todo!()
    }

    pub fn set_frequency(&mut self, freq: u32) {
        todo!()
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

impl<'d, T: Instance, M: Mode> embedded_hal::Pwm for Pwm<'d, T, M> {
    type Channel = Channel;
    type Duty = u16;
    type Time = u16;

    fn enable(&mut self, channel: Self::Channel) {
        todo!()
    }

    fn disable(&mut self, channel: Self::Channel) {
        todo!()
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        todo!()
    }

    fn get_max_duty(&self) -> Self::Duty {
        todo!()
    }

    fn get_period(&self) -> Self::Time {
        todo!()
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        todo!()
    }

    fn set_period<P>(&mut self, period: P)
    where
        P: Into<Self::Time>,
    {
        todo!()
    }
}

impl<'d, T: Instance> embedded_hal::PwmPin for PwmChannel<'d, T> {
    type Duty = u16;
    fn enable(&mut self) {
        todo!()
    }

    fn disable(&mut self) {
        todo!()
    }

    fn get_duty(&self) -> Self::Duty {
        todo!()
    }

    fn set_duty(&mut self, duty: Self::Duty) {
        todo!()
    }

    fn get_max_duty(&self) -> Self::Duty {
        todo!()
    }
}

use super::{
    oc::channel_event, Channel, ChannelType, Event, Instance, InputCaptureFilter,
    InputCapturePolarity, InputCapturePrescaler, InputCaptureSelection,
};
use core::marker::PhantomData;

/// 输入捕获通道配置
#[derive(Clone, Copy)]
pub struct InputCaptureConfig {
    /// 输入映射（CCxS）
    pub selection: InputCaptureSelection,
    /// 捕获边沿
    pub polarity: InputCapturePolarity,
    /// 输入预分频（每 N 个事件捕获一次）
    pub prescaler: InputCapturePrescaler,
    /// 数字滤波器
    pub filter: InputCaptureFilter,
}

impl Default for InputCaptureConfig {
    fn default() -> Self {
        Self {
            selection: InputCaptureSelection::DirectTI,
            polarity: InputCapturePolarity::Rising,
            prescaler: InputCapturePrescaler::Div1,
            filter: InputCaptureFilter::none(),
        }
    }
}

/// 通道过捕获事件
#[inline]
fn overcapture_event(channel: Channel) -> Event {
    match channel {
        Channel::CH1 => Event::CC1OF,
        Channel::CH2 => Event::CC2OF,
        Channel::CH3 => Event::CC3OF,
        Channel::CH4 => Event::CC4OF,
    }
}

/// 统一输入捕获
pub struct InputCapture<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance> InputCapture<'d, T> {
    pub(super) fn new() -> Self {
        Self { _t: PhantomData }
    }

    #[inline]
    fn check_channel(channel: Channel) {
        assert!(
            channel.index() < T::CHANNELS,
            "该定时器不支持此通道（可用通道数 = {}）",
            T::CHANNELS
        );
    }

    /// 配置输入捕获通道
    pub fn channel_config(&self, channel: Channel, config: InputCaptureConfig) {
        Self::check_channel(channel);
        T::set_channel_type(channel, ChannelType::Capture1);
        T::set_channel_input_config(
            channel,
            config.selection,
            config.polarity,
            config.prescaler,
            config.filter,
        );
        // 使能捕获
        T::set_enable_channel(channel, super::ChannelOutput::P, true);
    }

    /// 读取捕获值（CCR）
    #[inline]
    pub fn capture(&self, channel: Channel) -> u16 {
        Self::check_channel(channel);
        T::get_channel_capture(channel)
    }

    /// 是否已捕获
    #[inline]
    pub fn is_captured(&self, channel: Channel) -> bool {
        Self::check_channel(channel);
        T::event_flag(channel_event(channel))
    }

    /// 清除捕获标志
    #[inline]
    pub fn clear_capture(&self, channel: Channel) {
        Self::check_channel(channel);
        T::event_clear(channel_event(channel));
    }

    /// 是否发生过捕获
    #[inline]
    pub fn is_overcaptured(&self, channel: Channel) -> bool {
        Self::check_channel(channel);
        T::event_flag(overcapture_event(channel))
    }

    /// 清除过捕获标志
    #[inline]
    pub fn clear_overcapture(&self, channel: Channel) {
        Self::check_channel(channel);
        T::event_clear(overcapture_event(channel));
    }

    /// 设置周期（ARR），用于限制计数上限
    #[inline]
    pub fn set_period(&self, arr: u16) {
        T::set_auto_reload(arr);
    }

    /// 读取当前计数值
    #[inline]
    pub fn now(&self) -> u16 {
        T::get_cnt()
    }

    #[inline]
    pub fn start(&self) {
        T::start();
    }

    #[inline]
    pub fn stop(&self) {
        T::stop();
    }
}

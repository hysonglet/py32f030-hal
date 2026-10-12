use super::{
    Channel, ChannelConfig, ChannelOutput, ChannelType, Event, HasChannel, Instance,
    InputCaptureConfig, OnePulseMode, WithEncoder, WithOnePulse,
};
use core::marker::PhantomData;

/// 单脉冲配置
#[derive(Clone, Copy)]
pub struct OnePulseConfig {
    /// 输出通道
    pub output_channel: Channel,
    /// 输出通道配置
    pub channel: ChannelConfig,
    /// 计数周期（ARR）
    pub period: u16,
}

impl Default for OnePulseConfig {
    fn default() -> Self {
        Self {
            output_channel: Channel::CH1,
            channel: ChannelConfig {
                mode: super::ChannelMode::PWM1,
                ..Default::default()
            },
            period: 0xFFFF,
        }
    }
}

/// 单脉冲（One Pulse Mode）
///
/// 仅 TIM1 / TIM3 / TIM16 具备 OPM，因此受 `WithOnePulse` 编译期门控。
/// 外部触发输入需要 SMCR（即 `WithEncoder` 能力）与第二通道，故单独门控。
pub struct OnePulse<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance + WithOnePulse> OnePulse<'d, T> {
    pub(super) fn new(cfg: OnePulseConfig, op_mode: OnePulseMode) -> Self {
        assert!(
            cfg.output_channel.index() < T::CHANNELS,
            "该定时器不支持此输出通道"
        );

        T::stop();
        T::set_channel_type(cfg.output_channel, ChannelType::Pwm);
        T::set_channel_output_config(
            cfg.output_channel,
            cfg.channel.mode,
            cfg.channel.clear,
            cfg.channel.fast,
            cfg.channel.preload,
        );
        T::set_channel_compare(cfg.output_channel, cfg.channel.compare);
        T::set_channel_output_effective_level(cfg.output_channel, ChannelOutput::P, true, true, false);
        T::set_auto_reload(cfg.period);
        // OPM=1 时计数器在更新事件后停止
        T::enable_single_mode(op_mode == OnePulseMode::Single);

        Self { _t: PhantomData }
    }

    /// 启动单脉冲（软件触发）
    #[inline]
    pub fn start(&self) {
        T::event_clear(Event::UIF);
        T::start();
    }

    /// 停止
    #[inline]
    pub fn stop(&self) {
        T::stop();
    }

    /// 是否已完成（产生更新事件）
    #[inline]
    pub fn is_finished(&self) -> bool {
        T::event_flag(Event::UIF)
    }

    /// 清除完成标志
    #[inline]
    pub fn clear_finished(&self) {
        T::event_clear(Event::UIF);
    }

    /// 阻塞等待脉冲结束
    #[inline]
    pub fn wait(&self) {
        while !T::event_flag(Event::UIF) {}
        T::event_clear(Event::UIF);
    }
}

/// 外部触发启动（需要 SMCR + 第二通道，仅 TIM1 / TIM3）
impl<'d, T: Instance + WithOnePulse + WithEncoder + HasChannel<2>> OnePulse<'d, T> {
    /// 配置外部触发输入（CH2 作为 TI2 触发源）
    pub fn set_external_trigger(&self, config: InputCaptureConfig) {
        T::set_channel_input_config(
            Channel::CH2,
            config.selection,
            config.polarity,
            config.prescaler,
            config.filter,
        );
        // 触发模式：TRGI 上升沿启动计数器
        T::set_trigger_selection(super::TriggerSelection::TI2FP2);
        T::set_slave_mode(super::SlaveMode::Trigger);
    }
}

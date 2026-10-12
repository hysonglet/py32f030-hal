use super::{EncoderMode, HasChannel, Instance, SlaveMode, TriggerSelection, WithEncoder};
use core::marker::PhantomData;

/// 正交编码器接口
///
/// 仅 TIM1 / TIM3 具备从模式控制器（SMCR），因此所有方法都在
/// `WithEncoder` 约束下——对 TIM14 / TIM16 调用会编译失败。
pub struct Encoder<'d, T: Instance> {
    _t: PhantomData<&'d T>,
}

impl<'d, T: Instance + WithEncoder> Encoder<'d, T> {
    pub(super) fn new() -> Self {
        Self { _t: PhantomData }
    }

    /// 配置为正交编码器（CH1 / CH2 作为 TI1 / TI2 输入）
    pub fn config(&self, mode: EncoderMode) {
        // 编码器需要 CH1 与 CH2
        assert!(
            T::CHANNELS >= 2,
            "编码器模式需要至少 2 个通道（当前 = {}）",
            T::CHANNELS
        );
        T::set_encoder_mode(mode);
    }

    /// 设置计数上限（ARR）
    #[inline]
    pub fn set_period(&self, arr: u16) {
        T::set_auto_reload(arr);
    }

    /// 设置预分频
    #[inline]
    pub fn set_prescaler(&self, psc: u16) {
        T::set_prescaler(psc);
    }

    /// 读取当前计数值
    #[inline]
    pub fn now(&self) -> u16 {
        T::get_cnt()
    }

    /// 写入当前计数值
    #[inline]
    pub fn set_count(&self, cnt: u16) {
        T::set_cnt(cnt);
    }

    /// 启动
    #[inline]
    pub fn start(&self) {
        T::start();
    }

    /// 停止
    #[inline]
    pub fn stop(&self) {
        T::stop();
    }
}

/// 从模式相关（编码器所依赖的 SMCR 能力）
impl<'d, T: Instance + WithEncoder> Encoder<'d, T> {
    /// 设置从模式
    #[inline]
    pub fn set_slave_mode(&self, mode: SlaveMode) {
        T::set_slave_mode(mode);
    }

    /// 设置触发输入
    #[inline]
    pub fn set_trigger_selection(&self, ts: TriggerSelection) {
        T::set_trigger_selection(ts);
    }

    /// 使能主/从同步
    #[inline]
    pub fn set_master_slave_mode(&self, en: bool) {
        T::set_master_slave_mode(en);
    }
}

/// 编码器占用 CH1 / CH2：要求实例确实具备这两个通道
impl<'d, T: Instance + WithEncoder + HasChannel<1> + HasChannel<2>> Encoder<'d, T> {
    /// 确认该实例可用于双通道编码器输入
    #[inline]
    pub fn has_two_inputs(&self) -> bool {
        T::CHANNELS >= 2
    }
}

//! 定时器能力标记（capability traits）
//!
//! 每个 trait 只由**具备该硬件能力**的定时器实例实现。高层 API 把"定时器专有功能"
//! 放在以这些 trait 为约束的 `impl` 块中，从而在**编译期**就拒绝不支持的调用，
//! 而不是等到运行时 panic。
//!
//! 例如刹车/死区仅在 TIM1、TIM16 上存在，因此：
//! ```ignore
//! impl<'d, T: Instance + WithBreak> Pwm<'d, T> {
//!     pub fn enable_break(&self, cfg: BreakConfig) { .. }
//! }
//! ```
//! 对 TIM3 / TIM14 调用 `enable_break` 会直接编译失败。

use super::instance::sealed;

/// 定时器通道数量（编译期常量）
#[allow(private_bounds)]
pub trait ChannelCount: sealed::Instance {
    /// 该定时器实际可用的通道数
    const CHANNELS: usize;
}

/// 类型级通道标记：实例是否具备第 `N` 个通道（1 起）
///
/// 用于把"通道选择在编译期定死"，避免对单通道定时器请求 CH2 之类不存在的通道。
#[allow(private_bounds)]
pub trait HasChannel<const N: usize>: sealed::Instance {}

/// 具备互补输出（CHxN），仅 TIM1、TIM16
#[allow(private_bounds)]
pub trait WithComplementary: sealed::Instance {}

/// 具备刹车与死区（BDTR），仅 TIM1、TIM16
#[allow(private_bounds)]
pub trait WithBreak: sealed::Instance {}

/// 具备重复计数器（RCR），仅 TIM1、TIM16
#[allow(private_bounds)]
pub trait WithRepetition: sealed::Instance {}

/// 具备从模式控制器 / 编码器接口（SMCR），仅 TIM1、TIM3
#[allow(private_bounds)]
pub trait WithEncoder: sealed::Instance {}

/// 具备单脉冲模式（OPM），TIM1、TIM3、TIM16
#[allow(private_bounds)]
pub trait WithOnePulse: sealed::Instance {}

/// 具备中心对齐与向下计数（CR1.CMS / DIR），仅 TIM1、TIM3
#[allow(private_bounds)]
pub trait WithCenterAlign: sealed::Instance {}

/// 具备主模式触发输出 TRGO（CR2.MMS），仅 TIM1、TIM3
#[allow(private_bounds)]
pub trait WithMasterMode: sealed::Instance {}

/// 具备 TI1 输入重映射（OR），仅 TIM14
#[allow(private_bounds)]
pub trait WithTi1Remap: sealed::Instance {}

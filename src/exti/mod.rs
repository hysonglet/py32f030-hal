mod hal;
mod pins;

use core::convert::Infallible;
use core::{future::Future, marker::PhantomData, task::Poll};

use crate::gpio::Pin;
use crate::pac::interrupt;
use crate::{
    bit::*,
    gpio::{AnyPin, GpioPort, Input, PinLevel, PinPullUpDown, PinSpeed},
};

use critical_section::CriticalSection;
use embassy_hal_internal::{into_ref, Peripheral};
use embassy_sync::waitqueue::AtomicWaker;

use self::hal::sealed::Instance;

const EXIT_GPIO_COUNT: usize = 17;
#[allow(clippy::declare_interior_mutable_const)]
const ATOMIC_WAKE_CONST: AtomicWaker = AtomicWaker::new();
static EXIT_GPIO_WAKERS: [AtomicWaker; EXIT_GPIO_COUNT] = [ATOMIC_WAKE_CONST; EXIT_GPIO_COUNT];

impl Instance for Exti {}

struct Exti;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Line {
    // GPIO 0
    Line0 = 0,
    // GPIO 1
    Line1 = 1,
    // GPIO 2
    Line2 = 2,
    // GPIO 3
    Line3 = 3,
    // GPIO 4
    Line4 = 4,
    // GPIO 5
    Line5 = 5,
    // GPIO6
    Line6 = 6,
    // gpio 7
    Line7 = 7,
    // gpio 8
    Line8 = 8,
    // gpio 9
    Line9 = 9,
    // gpio 10
    Line10 = 10,
    // gpio 11
    Line11 = 11,
    // gpio 12
    Line12 = 12,
    // gpio 13
    Line13 = 13,
    // gpio 14
    Line14 = 14,
    // gpio 15
    Line15 = 15,
    // // PVD
    // Line16 = 16,
    // // COMP 1
    // Line17 = 17,
    // // COMP 2
    // Line18 = 18,
    // // RTC
    // Line19 = 19,
    // // LPTIM
    // Line29 = 29,
}

impl From<usize> for Line {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::Line0,
            1 => Self::Line1,
            2 => Self::Line2,
            3 => Self::Line3,
            4 => Self::Line4,
            5 => Self::Line5,
            6 => Self::Line6,
            7 => Self::Line7,
            8 => Self::Line8,
            9 => Self::Line9,
            10 => Self::Line10,
            11 => Self::Line11,
            12 => Self::Line12,
            13 => Self::Line13,
            14 => Self::Line14,
            15 => Self::Line15,
            _ => unreachable!(),
        }
    }
}

// pub trait ExitPin:
//     Peripheral<P = Self> + sealed::Instance + crate::gpio::Pin + 'static + Send
// {
//     fn set_pin_select(&self) {
//         Self::exit_channle_select(self.pin().into(), self.port().into());
//     }
// }

pub struct ExtiInput<'d> {
    pin: Input<'d>,
}

impl<'d> Unpin for ExtiInput<'d> {}

impl<'d> ExtiInput<'d> {
    pub fn new(
        pin: impl Peripheral<P = impl Pin> + 'd,
        pull: PinPullUpDown,
        speed: PinSpeed,
    ) -> Self {
        into_ref!(pin);

        Self {
            pin: Input::new(pin, pull, speed),
        }
    }

    /// 返回引脚的电平
    fn get_level(&self) -> PinLevel {
        self.pin.read()
    }

    /// 等待引脚电平变低
    pub async fn wait_for_low(&self) {
        let fut = ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::Falling);
        if self.get_level() == PinLevel::Low {
            return;
        }
        fut.await
    }

    /// 等待引脚电平变高
    pub async fn wait_for_high(&self) {
        let fut = ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::Rising);
        if self.get_level() == PinLevel::Hight {
            return;
        }
        fut.await
    }

    /// 等待上升沿信号
    pub async fn wait_for_rising_edge(&self) {
        let fut = ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::Rising);
        fut.await
    }

    /// 等待下降沿信号
    pub async fn wait_for_falling_edge(&self) {
        let fut = ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::Falling);
        fut.await
    }

    /// 等待上升沿和下降沿信号
    pub async fn wait_for_any_edge(&self) {
        let fut =
            ExtiInputFuture::new(self.pin.pin.port(), self.pin.pin.pin(), Edge::RisingFalling);
        fut.await
    }
}

impl<'a> embedded_hal::digital::v2::InputPin for ExtiInput<'a> {
    type Error = Infallible;
    fn is_low(&self) -> Result<bool, Self::Error> {
        self.pin.is_low()
    }

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.pin.is_high()
    }
}

struct BitIter(u32);

impl Iterator for BitIter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.trailing_zeros() {
            32 => None,
            idx => {
                self.0 = bit_mask_idx_clear::<1>(idx as usize, self.0);
                Some(idx)
            }
        }
    }
}

/// 信号边缘检测类型
#[derive(PartialEq)]
enum Edge {
    /// 上升沿,
    Rising,
    /// 下降沿
    Falling,
    /// 上升沿和下降沿
    RisingFalling,
}

impl Edge {
    fn is_rising(&self) -> bool {
        *self == Self::Rising || *self == Self::RisingFalling
    }

    fn is_falling(&self) -> bool {
        *self == Self::Falling || *self == Self::RisingFalling
    }
}

struct ExtiInputFuture<'a> {
    line: Line,
    edge: Edge,
    life: PhantomData<&'a mut AnyPin>,
}

impl<'a> ExtiInputFuture<'a> {
    fn new(port: GpioPort, pin: usize, edge: Edge) -> Self {
        let line: Line = pin.into();
        // line 选择
        Exti::exit_channle_select(line, port.into());

        critical_section::with(|_| {
            // 设置上升沿触发条件
            Exti::line_ring_edge(line, edge.is_rising());
            // 设置下降沿的触发条件
            Exti::line_falling_edge(line, edge.is_falling());

            // clear pending bit
            Exti::clear_pending(line);
            Exti::line_pend_enable(line, true);
        });

        Self {
            line,
            edge,
            life: PhantomData,
        }
    }
}

impl<'d> Future for ExtiInputFuture<'d> {
    type Output = ();
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        EXIT_GPIO_WAKERS[self.line as usize].register(cx.waker());

        if !Exti::is_line_pend_enable(self.line) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl<'d> Drop for ExtiInputFuture<'d> {
    fn drop(&mut self) {
        critical_section::with(|_| {
            if self.edge.is_rising() {
                Exti::line_ring_edge(self.line, false);
            } else if self.edge.is_falling() {
                Exti::line_falling_edge(self.line, false);
            }
            // Exit::line_falling_edge(self.line, false);
            // Exit::line_pend_enable(self.line, false);
        })
    }
}

#[interrupt]
fn EXTI0_1() {
    unsafe { on_gpio_line_irq(0x03) }
}

#[interrupt]
fn EXTI2_3() {
    unsafe { on_gpio_line_irq(0xc0) }
}

#[interrupt]
fn EXTI4_15() {
    unsafe { on_gpio_line_irq(0xfff0) }
}

unsafe fn on_gpio_line_irq(mask: u32) {
    let flag = Exti::block().pr.read().bits() & mask;
    for line in BitIter(flag) {
        Exti::line_pend_enable(Line::from(line as usize), false);
        Exti::clear_pending(Line::from(line as usize));
        EXIT_GPIO_WAKERS[line as usize].wake();
    }
}

pub(crate) fn init(_cs: CriticalSection) {
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt::EXTI4_15);
        cortex_m::peripheral::NVIC::unmask(interrupt::EXTI0_1);
        cortex_m::peripheral::NVIC::unmask(interrupt::EXTI2_3);
    }
}

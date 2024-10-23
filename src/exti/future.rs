use super::types::*;
use crate::exti::hal::sealed::Instance;
use crate::gpio::{AnyPin, GpioPort};
use crate::pac::interrupt;

use embassy_sync::waitqueue::AtomicWaker;

use core::{future::Future, marker::PhantomData, task::Poll};

const EXIT_GPIO_COUNT: usize = 17;
#[allow(clippy::declare_interior_mutable_const)]
const ATOMIC_WAKE_CONST: AtomicWaker = AtomicWaker::new();
static EXIT_GPIO_WAKERS: [AtomicWaker; EXIT_GPIO_COUNT] = [ATOMIC_WAKE_CONST; EXIT_GPIO_COUNT];

impl Instance for Exti {}

pub(crate) struct Exti;

pub struct ExtiInputFuture<'a> {
    line: Line,
    edge: Edge,
    life: PhantomData<&'a mut AnyPin>,
}

impl<'a> ExtiInputFuture<'a> {
    pub fn new(port: GpioPort, pin: usize, edge: Edge) -> Self {
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
    critical_section::with(|_cs| unsafe { on_gpio_line_irq(0x03) })
}

#[interrupt]
fn EXTI2_3() {
    critical_section::with(|_cs| unsafe { on_gpio_line_irq(0xc0) })
}

#[interrupt]
fn EXTI4_15() {
    critical_section::with(|_cs| unsafe { on_gpio_line_irq(0xfff0) })
}

unsafe fn on_gpio_line_irq(mask: u32) {
    let flag = Exti::block().pr.read().bits() & mask;
    for line in BitIter(flag) {
        Exti::line_pend_enable(Line::from(line as usize), false);
        Exti::clear_pending(Line::from(line as usize));
        EXIT_GPIO_WAKERS[line as usize].wake();
    }
}

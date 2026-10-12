use super::{Error, Event, Instance, TimerId};
use crate::mcu::peripherals::{TIM1, TIM14, TIM16, TIM17, TIM3};
use crate::pac::interrupt;
use core::{future::Future, marker::PhantomData, task::Poll};
use critical_section::CriticalSection;
use embassy_sync::waitqueue::AtomicWaker;
use enumset::EnumSet;

/// 每个定时器一个 Waker，按 `TimerId` 索引
/// （`AtomicWaker` 非 `Copy`，不能用 `[AtomicWaker::new(); N]` 简写）
static WAKER: [AtomicWaker; 5] = [
    AtomicWaker::new(),
    AtomicWaker::new(),
    AtomicWaker::new(),
    AtomicWaker::new(),
    AtomicWaker::new(),
];

pub struct EventFuture<T: Instance> {
    _t: PhantomData<T>,
    events: EnumSet<Event>,
}

impl<T: Instance> EventFuture<T> {
    pub fn new(events: EnumSet<Event>) -> Self {
        events.iter().for_each(|event| T::event_config(event, true));
        Self {
            _t: PhantomData,
            events,
        }
    }

    /// 中断函数调用
    #[inline]
    unsafe fn on_interrupt(_cs: CriticalSection, id: usize) {
        // 关闭已经发生的中断事件
        EnumSet::all().iter().for_each(|event| {
            if T::event_flag(event) {
                T::event_config(event, false);
            }
        });
        WAKER[id].wake()
    }
}

impl<T: Instance> Future for EventFuture<T> {
    type Output = Result<EnumSet<Event>, Error>;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        WAKER[T::id() as usize].register(cx.waker());

        let mut e = EnumSet::empty();
        for event in self.events {
            if T::event_flag(event) {
                T::event_clear(event);
                e |= event;
            }
        }

        if !e.is_empty() {
            return Poll::Ready(Ok(e));
        }

        Poll::Pending
    }
}

// TIM1 占用两个中断向量，两个都要唤醒同一个 Waker
#[interrupt]
fn TIM1_BRK_UP_TRG_COM() {
    critical_section::with(|cs| unsafe {
        EventFuture::<TIM1>::on_interrupt(cs, TimerId::TIM1 as usize)
    })
}

#[interrupt]
fn TIM1_CC() {
    critical_section::with(|cs| unsafe {
        EventFuture::<TIM1>::on_interrupt(cs, TimerId::TIM1 as usize)
    })
}

#[interrupt]
fn TIM3() {
    critical_section::with(|cs| unsafe {
        EventFuture::<TIM3>::on_interrupt(cs, TimerId::TIM3 as usize)
    })
}

#[interrupt]
fn TIM14() {
    critical_section::with(|cs| unsafe {
        EventFuture::<TIM14>::on_interrupt(cs, TimerId::TIM14 as usize)
    })
}

#[interrupt]
fn TIM16() {
    critical_section::with(|cs| unsafe {
        EventFuture::<TIM16>::on_interrupt(cs, TimerId::TIM16 as usize)
    })
}

#[interrupt]
fn TIM17() {
    critical_section::with(|cs| unsafe {
        EventFuture::<TIM17>::on_interrupt(cs, TimerId::TIM17 as usize)
    })
}

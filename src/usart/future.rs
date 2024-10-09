use super::{Error, Event, Id, Instance};
use crate::mcu::peripherals::{USART1, USART2};
use crate::pac::interrupt;
use core::{future::Future, marker::PhantomData, task::Poll};
use critical_section::CriticalSection;

use embassy_sync::waitqueue::AtomicWaker;
use enumset::EnumSet;

const _ATOMIC_WAKER: AtomicWaker = AtomicWaker::new();
const _EVENT_COUNT: usize = Event::PE as usize;
const _WAKER_COUNT: usize = Id::USART2 as usize;
static EVENT_WAKERS: [[AtomicWaker; _EVENT_COUNT]; _WAKER_COUNT] =
    [[_ATOMIC_WAKER; _EVENT_COUNT]; _WAKER_COUNT];

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
            /* 匹配到中断了 */
            if T::is_event_enable(event) && T::event_flag(event) {
                // 关闭触发的中断，防止重复响应
                T::event_config(event, false);
                EVENT_WAKERS[event as usize][id].wake()
            }
        });
    }
}

impl<T: Instance> Future for EventFuture<T> {
    type Output = Result<(), Error>;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        self.events.iter().for_each(|e| {
            EVENT_WAKERS[e as usize][T::id() as usize].register(cx.waker());
        });

        let mut events = EnumSet::empty();
        // 消除所有关注的中断标志
        for event in self.events {
            if T::event_flag(event) {
                T::event_clear(event);
                events |= event;
            }
        }

        if !events.is_empty() {
            return Poll::Ready(Ok(()));
        }
        // 没有任何事件
        Poll::Pending
    }
}

#[interrupt]
fn USART1() {
    critical_section::with(|cs| unsafe {
        EventFuture::<USART1>::on_interrupt(cs, Id::USART1 as usize)
    })
}

#[interrupt]
fn USART2() {
    critical_section::with(|cs| unsafe {
        EventFuture::<USART2>::on_interrupt(cs, Id::USART2 as usize)
    })
}

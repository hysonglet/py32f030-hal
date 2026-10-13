use super::{Event, Id, Instance};
use crate::mcu::peripherals::{USART1, USART2};
use crate::pac::interrupt;
use core::{future::Future, marker::PhantomData, task::Poll};
use critical_section::CriticalSection;

#[cfg(feature = "embassy")]
use embassy_sync::waitqueue::AtomicWaker;
use enumset::EnumSet;

#[cfg(feature = "embassy")]
const _EVENT_COUNT: usize = Event::PE as usize + 1;
#[cfg(feature = "embassy")]
const _WAKER_COUNT: usize = Id::USART2 as usize + 1;
#[cfg(feature = "embassy")]
pub(super) static EVENT_WAKERS: [AtomicWaker; _EVENT_COUNT * _WAKER_COUNT] =
    [const { AtomicWaker::new() }; _EVENT_COUNT * _WAKER_COUNT];

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

    /// Called from the interrupt handler.
    #[inline]
    unsafe fn on_interrupt(_cs: CriticalSection, id: usize) {
        EnumSet::all().iter().for_each(|event| {
            if T::is_event_enable(event) && T::event_flag(event) {
                // Disable the triggered interrupt to avoid repeated responses.
                T::event_config(event, false);
                EVENT_WAKERS[id * _EVENT_COUNT + event as usize].wake()
            }
        });
    }
}

impl<T: Instance> Future for EventFuture<T> {
    type Output = EnumSet<Event>;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let mut events = EnumSet::empty();
        // Clear all watched interrupt flags.
        for event in self.events {
            if T::event_flag(event) {
                T::event_clear(event);
                events |= event;
            }
        }

        if !events.is_empty() {
            return Poll::Ready(events);
        }

        self.events.iter().for_each(|e| {
            EVENT_WAKERS[T::id() as usize * _EVENT_COUNT + e as usize].register(cx.waker());
        });
        // No events occurred.
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

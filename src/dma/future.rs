use super::{Channel, Event, Instance};
use crate::pac::interrupt;
use crate::{clock::peripheral::PeripheralInterrupt, mcu::peripherals::DMA};
use core::{future::Future, marker::PhantomData, task::Poll};
use critical_section::CriticalSection;

#[cfg(feature = "embassy")]
use embassy_sync::waitqueue::AtomicWaker;
use enumset::EnumSet;

#[allow(clippy::declare_interior_mutable_const)]
#[cfg(feature = "embassy")]
const _ATOMIC_WAKER: AtomicWaker = AtomicWaker::new();
#[cfg(feature = "embassy")]
const _WAKER_COUNT: usize = 3;
#[cfg(feature = "embassy")]
pub(super) static EVENT_WAKERS: [AtomicWaker; _WAKER_COUNT] = [_ATOMIC_WAKER; _WAKER_COUNT];

pub struct EventFuture<T: Instance> {
    _t: PhantomData<T>,
    events: EnumSet<Event>,
    channel: Channel,
}

impl<T: Instance> Unpin for EventFuture<T> {}

impl<T: Instance> Drop for EventFuture<T> {
    fn drop(&mut self) {
        // Disable the channel interrupt.
        self.channel.disable_interrupt();
    }
}

impl<T: Instance> EventFuture<T> {
    pub fn new(channel: Channel, events: EnumSet<Event>) -> Self {
        Self {
            _t: PhantomData,
            events,
            channel,
        }
    }

    /// Called from the interrupt handler.
    #[inline]
    unsafe fn on_interrupt(_cs: CriticalSection, channel: Channel, events: EnumSet<Event>) {
        events.iter().for_each(|event| {
            /* the interrupt is enabled and the interrupt was matched */
            if T::is_event_enable(channel, event) && T::event_flag(channel, event) {
                // Disable the triggered interrupt to avoid repeated responses.
                T::event_config(channel, event, false);
                EVENT_WAKERS[channel as usize].wake()
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
            if T::event_flag(self.channel, event) {
                T::event_clear(self.channel, event);
                events |= event;
            }
        }

        if !events.is_empty() {
            return Poll::Ready(events);
        }

        EVENT_WAKERS[self.channel as usize].register(cx.waker());
        events
            .iter()
            .for_each(|event| T::event_config(self.channel, event, true));
        // Enable the channel interrupt.
        self.channel.enable_interrupt();
        // No events occurred.
        Poll::Pending
    }
}

#[interrupt]
fn DMA_CHANNEL1() {
    critical_section::with(|cs| unsafe {
        EventFuture::<DMA>::on_interrupt(cs, Channel::Channel1, EnumSet::all())
    })
}

#[interrupt]
fn DMA_CHANNEL2_3() {
    // Channel 2 and Channel 3 may share a single vector, so iterate over both.
    critical_section::with(|cs| unsafe {
        EventFuture::<DMA>::on_interrupt(cs, Channel::Channel2, EnumSet::all());
        EventFuture::<DMA>::on_interrupt(cs, Channel::Channel3, EnumSet::all())
    })
}

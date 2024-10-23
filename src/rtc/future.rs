use super::*;
use crate::mcu::peripherals::RTC;
use crate::pac::interrupt;
use core::{future::Future, marker::PhantomData, task::Poll};
use embassy_sync::waitqueue::AtomicWaker;

static WAKER: [AtomicWaker; 1] = [AtomicWaker::new()];

pub struct WakeFuture<T: Instance> {
    _t: PhantomData<T>,
    event: EnumSet<EventKind>,
}

impl<T: Instance> WakeFuture<T> {
    pub fn new(event: EnumSet<EventKind>) -> Self {
        Self {
            _t: PhantomData,
            event,
        }
    }

    #[inline]
    fn on_interrupt() {
        EnumSet::all().iter().for_each(|event| {
            if T::event_flag(event) && T::is_enable_interrupt(event) {
                T::event_config(event, false);
            }
        });
        WAKER[T::id() as usize].wake()
    }
}

impl<T: Instance> Future for WakeFuture<T> {
    type Output = ();
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        WAKER[T::id() as usize].register(cx.waker());

        let mut interrupt = false;

        self.event.iter().for_each(|event| {
            if T::event_flag(event) {
                interrupt = true;
                T::clear_interrupt(event);
            }
        });

        if interrupt {
            T::disable_config();
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl<T: Instance> Drop for WakeFuture<T> {
    fn drop(&mut self) {}
}

#[interrupt]
fn RTC() {
    critical_section::with(|_cs| WakeFuture::<RTC>::on_interrupt())
}

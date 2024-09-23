use super::{Error, Event, Instance};
use crate::{mcu::peripherals::TIM1, pac::interrupt};
use core::{future::Future, marker::PhantomData, task::Poll};
use embassy_sync::waitqueue::AtomicWaker;
use enumset::EnumSet;

const ATOMICWAKE: AtomicWaker = AtomicWaker::new();
static WAKER: [AtomicWaker; 1] = [ATOMICWAKE; 1];

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
    unsafe fn on_interrupt() {
        // 关闭已经发生的中断事件
        EnumSet::all().iter().for_each(|event| {
            /* 匹配到中断了 */
            if T::event_flag(event) {
                // 清除标志
                T::event_clear(event);
                // 关闭该中断
                T::event_config(event, false);
            }
        });
        WAKER[T::id() as usize].wake()
    }
}

impl<T: Instance> Future for EventFuture<T> {
    type Output = Result<(), Error>;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        WAKER[T::id() as usize].register(cx.waker());

        for event in self.events {
            if T::event_flag(event) {
                T::event_clear(event);
                return Poll::Ready(Ok(()));
            }
        }

        Poll::Pending
    }
}

#[interrupt]
fn TIM1_BRK_UP_TRG_COM() {
    critical_section::with(|_cs| unsafe { EventFuture::<TIM1>::on_interrupt() })
}

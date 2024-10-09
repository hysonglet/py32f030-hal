use super::{Error, Event, Instance};
use crate::mcu::peripherals::{USART1, USART2};
use crate::pac::interrupt;
use core::{future::Future, marker::PhantomData, task::Poll};
use critical_section::{CriticalSection, Mutex};
// use embassy_sync::blocking_mutex::{Mutex, raw::CriticalSectionRawMutex};
use embassy_sync::waitqueue::AtomicWaker;
use enumset::EnumSet;

static RX_WAKER: [AtomicWaker; 1] = [AtomicWaker::new(); 1];
static TX_WAKER: [AtomicWaker; 1] = [AtomicWaker::new(); 1];

static mut EVENTS: Mutex<EnumSet<Event>> = Mutex::new(EnumSet::empty());

pub struct EventFuture<T: Instance> {
    _t: PhantomData<T>,
    // events: EnumSet<Event>,
}

impl<T: Instance> EventFuture<T> {
    pub fn new(events: EnumSet<Event>) -> Self {
        events.iter().for_each(|event| T::event_config(event, true));
        Self {
            _t: PhantomData,
            // events,
        }
    }

    /// 中断函数调用
    #[inline]
    unsafe fn on_interrupt(cs: CriticalSection) {
        let events = EVENTS.borrow(cs);
        // 关闭已经发生的中断事件
        EnumSet::all().iter().for_each(|event| {
            /* 匹配到中断了 */
            if T::event_flag(event) {
                // 关闭触发的中断，防止重复响应
                T::event_config(event, false);
            }
        });

        if events.contains(Event::RXNE) {
            RX_WAKER[T::id() as usize].wake()
        }

        if events.contains(Event::TXE) | events.contains(Event::TC) {
            TX_WAKER[T::id() as usize].wake();
        }
    }
}

impl<T: Instance> Future for EventFuture<T> {
    type Output = Result<(), Error>;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let mut others_event = EnumSet::empty();
        let mut err_event = EnumSet::empty();

        // 注册相应的唤醒条件
        let events = critical_section::with(|cs| unsafe { EVENTS.borrow(cs) });

        // 消除所有关注的中断标志
        for event in *events {
            if (Event::TC | Event::TXE).contains(event) {
                TX_WAKER[T::id() as usize].register(cx.waker());
            } else if Event::RXNE == event {
                RX_WAKER[T::id() as usize].register(cx.waker());
            }
            // 其他标志可能发生错误
            // todo
            if T::event_flag(event) {
                T::event_clear(event);
                if (Event::ORE | Event::NE | Event::FE | Event::PE).contains(event) {
                    // 检测到错误了
                    err_event |= event;
                } else {
                    // 检测到收发事件
                    others_event |= event;
                }
            }
        }

        // 有错误产生
        if !err_event.is_empty() {
            return Poll::Ready(Err(Error::Others));
        } else if !others_event.is_empty() {
            return Poll::Ready(Ok(()));
        }

        // 没有任何事件
        Poll::Pending
    }
}

#[interrupt]
fn USART1() {
    critical_section::with(|cs| unsafe { EventFuture::<USART1>::on_interrupt(cs) })
}

#[interrupt]
fn USART2() {
    critical_section::with(|cs| unsafe { EventFuture::<USART2>::on_interrupt(cs) })
}

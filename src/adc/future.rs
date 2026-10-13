use enumset::EnumSet;

use super::*;
use crate::pac::interrupt;

pub struct ChannelInputFuture<T: Instance> {
    _channel: AdcChannel,
    _t: PhantomData<T>,
}

impl<T: Instance> ChannelInputFuture<T> {
    /// Create an EOC interrupt.
    /// Enable the ADC global interrupt beforehand; the task is suspended while awaiting.
    pub fn new_with_channel(channel: AdcChannel) -> Self {
        T::event_clear(Event::EOC);
        T::event_config(Event::EOC, true);

        // Enable the channel.
        T::channel_enable_exclusive(channel);

        T::start();
        Self {
            _channel: channel,
            _t: PhantomData,
        }
    }
    #[inline]
    unsafe fn on_interrupt() {
        EnumSet::all().iter().for_each(|event| {
            if T::event_flag(event) {
                T::event_config(event, false);
            }
        });
        ADC_INT_WAKER[T::id() as usize].wake()
    }
}

impl<T: Instance> Future for ChannelInputFuture<T> {
    type Output = u16;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        ADC_INT_WAKER[T::id() as usize].register(cx.waker());

        if T::event_flag(Event::EOC) {
            // Reading DR automatically clears the EOC bit.
            Poll::Ready(T::data_read())
        } else {
            Poll::Pending
        }
    }
}

impl<T: Instance> Drop for ChannelInputFuture<T> {
    fn drop(&mut self) {
        // T::event_config(Event::EOC, false);
    }
}

// impl<T: Instance> Unpin for ChannelInput<T> {}

#[interrupt]
fn ADC_COMP() {
    // ADC1 EOC interrupt
    critical_section::with(|_cs| unsafe {
        ChannelInputFuture::<ADC>::on_interrupt();
    })
    // TODO!
    // COMP interrupt
}

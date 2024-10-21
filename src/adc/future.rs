use enumset::EnumSet;

use super::*;
use crate::pac::interrupt;

pub struct ChannelInputFuture<T: Instance> {
    _channel: AdcChannel,
    _t: PhantomData<T>,
}

impl<T: Instance> ChannelInputFuture<T> {
    /// 新建一个 eoc 中断
    /// 记得提前打开 ADC 的总中断，改任务会暂停在 异步中
    pub fn new_with_channel(channel: AdcChannel) -> Self {
        T::event_clear(Event::EOC);
        T::event_config(Event::EOC, true);

        // 开启通道
        T::channel_enable_exclusive(channel);

        T::start();
        // // 软件触发，则先触发一次
        // if T::is_soft_trigle() {}

        Self {
            _channel: channel,
            _t: PhantomData,
        }
    }
    #[inline]
    unsafe fn on_interrupt() {
        // 关闭已经发生的中断事件
        EnumSet::all().iter().for_each(|event| {
            /* 匹配到中断了 */
            if T::event_flag(event) {
                // defmt::info!("{:x}", event as u32);
                // 关闭该中断
                T::event_config(event, false);
            }
        });
        // defmt::info!("XXXXX");
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
            // 读取 dr 寄存器会自动清除 eoc 位
            Poll::Ready(T::data_read())
        } else {
            Poll::Pending
        }
    }
}

impl<T: Instance> Drop for ChannelInputFuture<T> {
    fn drop(&mut self) {
        // 关闭中断
        // T::event_config(Event::EOC, false);
    }
}

// impl<T: Instance> Unpin for ChannelInput<T> {}

#[interrupt]
fn ADC_COMP() {
    // ADC1 的中断 eoc
    critical_section::with(|_cs| unsafe {
        ChannelInputFuture::<ADC>::on_interrupt();
    })
    // TODO!
    // comp 的中断
}

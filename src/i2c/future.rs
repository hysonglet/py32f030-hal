use core::marker::PhantomData;
use crate::{mcu::peripherals::I2C, pac::interrupt};
use embassy_sync::waitqueue::AtomicWaker;
use enumset::{EnumSet, EnumSetType};
use super::Instance;

const ATOMICWAKE: AtomicWaker = AtomicWaker::new();
static WAKER: [AtomicWaker; 1] = [ATOMICWAKE; 1];

#[derive(EnumSetType)]
pub enum Event {
    /// 起始位已发送(Master)                        // 开启控制bit: ITEVTEN
    SB, 
    /// 地址已发送(Master) 或 地址匹配(Slave)
    ADD,
    /// 已收到停止(Slave)
    STOPF,
    /// 数据字节传输完成
    BTF,

    /// 接收缓冲区非空                              // 开启控制 bit: ITEVTEN 和ITBUFEN
    RXNE,
    /// 发送缓冲区空
    TXE,

    /// 总线错误                                   // 开启控制 bit: ITERREN
    BERR,
    /// 仲裁丢失(Master)
    ARLO,
    /// 响应失败
    AF,
    /// 过载/欠载
    OVR,
    /// PEC错误
    PECERR,
}

impl Event {
    const fn count() -> usize {
        11
    }
}

pub struct EventFuture<T: Instance>{
    _t: PhantomData<T>,
    event: EnumSet<Event>,
}

impl<T:Instance> EventFuture<T> {
    pub fn new(event: EnumSet<Event>) -> Self {
        Self {
            _t: PhantomData,
            event
        }
    }

    /// 中断函数调用
    #[inline]
    unsafe fn on_interrupt() {
        EnumSet::all().iter().for_each(|event| {
            /* 匹配到中断了 */
            if T::is_event_match(event) {
                // 关闭该中断
                T::event_config(event, false);
            }
        });
        WAKER[T::id() as usize].wake()
    }
}

#[interrupt]
fn I2C1() {
    critical_section::with(|_cs| unsafe {
        EventFuture::<I2C>::on_interrupt()
    })
}
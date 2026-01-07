use crate::pac::interrupt;
use core::cell::RefCell;
use cortex_m::interrupt::{free, CriticalSection, InterruptNumber, Mutex};

/// 中断处理错误的类型
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidInterruptNumber,
    DoubleBinding,
}

/// 定义中断处理函数的类型
type InterruptHandle = &'static dyn Fn(&CriticalSection);

const INTERRUPT_HANDLE_CNT: usize = 32;

/// 定义中断处理函数的数组
static mut INTERRUPT_HANDLERS: [Mutex<RefCell<Option<InterruptHandle>>>; INTERRUPT_HANDLE_CNT] = {
    const INIT_HANDLER: Mutex<RefCell<Option<InterruptHandle>>> = Mutex::new(RefCell::new(None));
    [INIT_HANDLER; INTERRUPT_HANDLE_CNT]
};

pub trait BindInterrupt: InterruptNumber + Copy + Clone {
    #[cfg(feature = "embassy")]
    /// 绑定一个默认的中断处理函数给funtures用
    fn bind_default(&self) -> Result<(), Error>;

    /// 绑定一个中断处理函数
    fn bind(&self, f: &'static dyn Fn(&CriticalSection)) -> Result<(), Error> {
        bind(self.number() as usize, f)
    }

    /// 解绑中断处理函数
    fn unbind(&self) {
        let _ = unbind(self.number() as usize);
    }

    /// 开启外设的总中断
    fn enable_irq(&self) {
        unsafe { cortex_m::peripheral::NVIC::unmask(*self) }
    }

    /// 关闭外设的总中断
    fn disable_irq(&self) {
        cortex_m::peripheral::NVIC::mask(*self)
    }
}

pub(super) fn bind(irq_num: usize, f: InterruptHandle) -> Result<(), Error> {
    if irq_num >= 32 {
        return Err(Error::InvalidInterruptNumber);
    }

    free(|cs| {
        let mut handler_cell = unsafe { INTERRUPT_HANDLERS[irq_num].borrow(cs).borrow_mut() };
        if handler_cell.is_some() {
            Err(Error::DoubleBinding)
        } else {
            *handler_cell = Some(f);
            Ok(())
        }
    })
}

pub(super) fn unbind(irq_num: usize) -> Result<(), Error> {
    if irq_num >= 32 {
        return Err(Error::InvalidInterruptNumber);
    }

    free(|cs| {
        let mut handler_cell =
            unsafe { INTERRUPT_HANDLERS[irq_num as usize].borrow(cs).borrow_mut() };
        *handler_cell = None;
    });
    Ok(())
}

pub fn enable<I: InterruptNumber>(irq: I) {
    unsafe { cortex_m::peripheral::NVIC::unmask(irq) }
}

pub fn disable<I: InterruptNumber>(irq: I) {
    cortex_m::peripheral::NVIC::mask(irq)
}

// 宏定义中断处理函数
macro_rules! define_interrupt_wrapper {
    ($irq_name:ident, $irq_num:expr) => {
        #[interrupt]
        fn $irq_name() {
            free(|cs| {
                let handler_cell = unsafe { INTERRUPT_HANDLERS[$irq_num].borrow(cs).borrow_mut() };

                if let Some(handler) = *handler_cell {
                    handler(cs);
                }
            });
        }
    };
}
// 定义所有的中断处理函数
define_interrupt_wrapper!(WWDG, 0);
define_interrupt_wrapper!(PVD, 1);
define_interrupt_wrapper!(RTC, 2);
define_interrupt_wrapper!(FLASH, 3);
define_interrupt_wrapper!(RCC, 4);
define_interrupt_wrapper!(EXTI0_1, 5);
define_interrupt_wrapper!(EXTI2_3, 6);
define_interrupt_wrapper!(EXTI4_15, 7);
define_interrupt_wrapper!(DMA_CHANNEL1, 9);
define_interrupt_wrapper!(DMA_CHANNEL2_3, 10);
define_interrupt_wrapper!(ADC_COMP, 12);
define_interrupt_wrapper!(TIM1_BRK_UP_TRG_COM, 13);
define_interrupt_wrapper!(TIM1_CC, 14);
define_interrupt_wrapper!(TIM3, 16);
define_interrupt_wrapper!(TIM14, 19);
define_interrupt_wrapper!(TIM16, 21);
define_interrupt_wrapper!(TIM17, 22);
define_interrupt_wrapper!(I2C1, 23);
define_interrupt_wrapper!(SPI1, 25);
define_interrupt_wrapper!(SPI2, 26);
define_interrupt_wrapper!(USART1, 27);
define_interrupt_wrapper!(USART2, 28);
define_interrupt_wrapper!(LED, 30);

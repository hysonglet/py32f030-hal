use crate::pac::interrupt;

pub(super) static mut CLOSURE: Option<*const dyn Fn()> = None;

pub fn dispatch() {
    unsafe {
        if let Some(func) = CLOSURE {
            (*func)()
        }
    }
}

// ADC 中断服务函数
#[interrupt]
fn ADC_COMP() {
    // ADC1 的中断 eoc
    critical_section::with(|_cs| {
        dispatch();
    })
}

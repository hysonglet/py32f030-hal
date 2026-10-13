use crate::pac::interrupt;

pub(super) static mut CLOSURE: Option<*const dyn Fn()> = None;

pub fn dispatch() {
    unsafe {
        if let Some(func) = CLOSURE {
            (*func)()
        }
    }
}

// ADC interrupt service routine
#[interrupt]
fn ADC_COMP() {
    // ADC1 EOC interrupt
    critical_section::with(|_cs| {
        dispatch();
    })
}

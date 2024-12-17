use alloc::boxed::Box;

pub fn register(closure: &'static mut Option<*const dyn Fn()>, f: Box<dyn Fn()>) {
    unsafe {
        if let Some(old) = *closure {
            *closure = None;
            let _ = alloc::boxed::Box::from_raw(old as *mut dyn Fn());
        }
        let raw = alloc::boxed::Box::into_raw(f);
        *closure = Some(raw)
    }
}

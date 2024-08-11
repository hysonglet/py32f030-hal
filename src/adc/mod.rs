mod hal;
mod pins;

// use crate::macro_def::pin_af_for_instance_def;

use embassy_hal_internal::Peripheral;

pub trait Instance: Peripheral<P = Self> + hal::sealed::Instance + 'static + Send {}

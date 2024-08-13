#![macro_use]

pub(crate) use super::impl_pin_af;
pub(crate) use super::impl_pin_analog;
pub(crate) use super::pin_af_for_instance_def;

#[macro_export]
macro_rules! pin_af_for_instance_def {
    (
        $pin_trait_name: ident, $instance: ident
    ) => {
        pub trait $pin_trait_name<T: $instance>: crate::gpio::Pin {
            fn af(&self) -> crate::gpio::PinAF;

            fn set_instance_af(
                &self,
                speed: crate::gpio::PinSpeed,
                io_type: crate::gpio::PinIoType,
            ) {
                self.set_mode(crate::gpio::PinMode::Af);
                self.set_af(self.af());
                self.set_speed(speed);
                self.set_io_type(io_type);
            }
        }
    };
}

#[macro_export]
macro_rules! impl_pin_af {
    (
        $pin_port: ident, $gpio_pin_name: ident, $instance: ident, $function_trait: ident, $af: ident
    ) => {
        impl $function_trait<peripherals::$instance> for $pin_port::$gpio_pin_name {
            fn af(&self) -> gpio::PinAF {
                gpio::PinAF::$af
            }
        }
    };
}

#[macro_export]
macro_rules! impl_pin_analog {
    (
        $pin_port: ident, $gpio_pin_name: ident, $instance: ident, $function_trait: ident
    ) => {
        impl $function_trait<peripherals::$instance> for $pin_port::$gpio_pin_name {
            fn analog_channel(&self) {
                todo!()
            }
        }
    };
}

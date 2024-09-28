#![macro_use]

pub(crate) use super::impl_pin_af;
pub(crate) use super::impl_sealed_peripheral_id;
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
                defmt::info!("{:?}", self.af() as u8);
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

// sealed peripheral id impl
#[macro_export]
macro_rules! impl_sealed_peripheral_id {
    (
        $peripheral: ident, $id: ident
    ) => {
        impl Instance for crate::mcu::peripherals::$peripheral {}

        impl hal::sealed::Instance for crate::mcu::peripherals::$peripheral {
            fn id() -> Id {
                Id::$id
            }
        }
    };
}

use super::{AdcChannel, AnalogPin};
use crate::gpio::hal::sealed::Pin;
use crate::gpio::{gpioa, gpiob};
use crate::mcu::peripherals;

macro_rules! impl_pin_analog {
    (
        $pin_port: ident, $gpio_pin_name: ident, $instance: ident, $function_trait: ident, $channel: ident
    ) => {
        impl $function_trait<peripherals::$instance> for $pin_port::$gpio_pin_name {
            fn channel(&self) -> AdcChannel {
                AdcChannel::$channel
            }

            fn as_anlog(&self) {
                self.set_mode(crate::gpio::PinMode::Analog);
                self.set_io_type(crate::gpio::PinIoType::Floating);
            }
        }
    };
}

impl_pin_analog!(gpioa, PA0, ADC, AnalogPin, Channel0);
impl_pin_analog!(gpioa, PA1, ADC, AnalogPin, Channel1);
impl_pin_analog!(gpioa, PA2, ADC, AnalogPin, Channel2);
impl_pin_analog!(gpioa, PA3, ADC, AnalogPin, Channel3);
impl_pin_analog!(gpioa, PA4, ADC, AnalogPin, Channel4);
impl_pin_analog!(gpioa, PA5, ADC, AnalogPin, Channel5);
impl_pin_analog!(gpioa, PA6, ADC, AnalogPin, Channel6);
impl_pin_analog!(gpioa, PA7, ADC, AnalogPin, Channel7);

impl_pin_analog!(gpiob, PB0, ADC, AnalogPin, Channel8);
impl_pin_analog!(gpiob, PB1, ADC, AnalogPin, Channel9);

pub struct TemperatureChannel;
pub struct VRrefChannel;

impl AnalogPin<peripherals::ADC> for TemperatureChannel {
    fn channel(&self) -> AdcChannel {
        AdcChannel::Channel11
    }

    fn as_anlog(&self) {}
}

impl AnalogPin<peripherals::ADC> for VRrefChannel {
    fn channel(&self) -> AdcChannel {
        AdcChannel::Channel12
    }

    fn as_anlog(&self) {}
}

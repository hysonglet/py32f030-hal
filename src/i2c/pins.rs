use super::{SclPin, SdaPin};
use crate::gpio::{self, gpioa, gpiob, gpiof};
use crate::macro_def::impl_pin_af;
use crate::mcu::peripherals;

// Associate a pin function, generating the pin trait bound to a peripheral function.
// Implement the corresponding function trait for all pins with I2C capability.
impl_pin_af!(gpiof, PF0_OSC_IN, I2C, SdaPin, AF12);
impl_pin_af!(gpiof, PF1_OSC_OUT, I2C, SclPin, AF12);

impl_pin_af!(gpioa, PA2, I2C, SdaPin, AF12);
// PA3's I2C_SCL alternate function is AF12 (previously missing, so the PA2+PA3 combo failed to compile).
impl_pin_af!(gpioa, PA3, I2C, SclPin, AF12);

impl_pin_af!(gpioa, PA7, I2C, SdaPin, AF12);
impl_pin_af!(gpioa, PA8, I2C, SclPin, AF12);

impl_pin_af!(gpioa, PA9, I2C, SdaPin, AF12);
impl_pin_af!(gpioa, PA9, I2C, SclPin, AF6);

impl_pin_af!(gpioa, PA10, I2C, SdaPin, AF6);
impl_pin_af!(gpioa, PA10, I2C, SclPin, AF12);

impl_pin_af!(gpioa, PA11, I2C, SclPin, AF6);
impl_pin_af!(gpioa, PA12, I2C, SdaPin, AF6);

impl_pin_af!(gpiob, PB6, I2C, SclPin, AF6);
impl_pin_af!(gpiob, PB7, I2C, SdaPin, AF6);
impl_pin_af!(gpiob, PB8, I2C, SdaPin, AF12);
impl_pin_af!(gpiob, PB8, I2C, SclPin, AF6);

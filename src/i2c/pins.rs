use super::{SclPin, SdaPin};
use crate::gpio::{self, gpioa, gpiob, gpiof};
use crate::macro_def::impl_pin_af;
use crate::mcu::peripherals;

// 指定 引脚功能，生成与外设功能绑定的引脚 trait
// 为所有具有 I2c 功能的引脚实现相应功能的 trait
impl_pin_af!(gpiof, PF0_OSC_IN, I2C, SdaPin, AF12);
impl_pin_af!(gpiof, PF1_OSC_OUT, I2C, SclPin, AF12);

impl_pin_af!(gpioa, PA2, I2C, SdaPin, AF12);
impl_pin_af!(gpioa, PA3, I2C, SclPin, AF12);

impl_pin_af!(gpioa, PA7, I2C, SdaPin, AF12);
impl_pin_af!(gpioa, PA8, I2C, SclPin, AF12);

impl_pin_af!(gpioa, PA9, I2C, SdaPin, AF6);
impl_pin_af!(gpioa, PA9, I2C, SclPin, AF6);

impl_pin_af!(gpioa, PA10, I2C, SdaPin, AF6);
impl_pin_af!(gpioa, PA10, I2C, SclPin, AF12);

impl_pin_af!(gpioa, PA11, I2C, SclPin, AF6);
impl_pin_af!(gpioa, PA12, I2C, SdaPin, AF6);

impl_pin_af!(gpiob, PB6, I2C, SclPin, AF6);
impl_pin_af!(gpiob, PB7, I2C, SdaPin, AF6);
impl_pin_af!(gpiob, PB8, I2C, SdaPin, AF12);
impl_pin_af!(gpiob, PB8, I2C, SclPin, AF6);

use super::{
    TimerBkInPin, TimerChannel1NPin, TimerChannel1Pin, TimerChannel2NPin, TimerChannel2Pin,
    TimerChannel3NPin, TimerChannel3Pin, TimerChannel4Pin, TimerEtrPin,
};
use crate::gpio::{self, gpioa, gpiob, gpiof};
use crate::macro_def::impl_pin_af;
use crate::mcu::peripherals;

impl_pin_af!(gpioa, PA0, TIM1, TimerChannel3Pin, AF13);
impl_pin_af!(gpioa, PA0, TIM1, TimerChannel1NPin, AF14);
impl_pin_af!(gpioa, PA1, TIM1, TimerChannel4Pin, AF13);
impl_pin_af!(gpioa, PA1, TIM1, TimerChannel2NPin, AF14);
impl_pin_af!(gpioa, PA3, TIM1, TimerChannel1Pin, AF13);
impl_pin_af!(gpioa, PA6, TIM1, TimerBkInPin, AF2);
impl_pin_af!(gpioa, PA8, TIM1, TimerChannel1Pin, AF2);
impl_pin_af!(gpioa, PA9, TIM1, TimerChannel2Pin, AF2);
impl_pin_af!(gpioa, PA9, TIM1, TimerBkInPin, AF13);
impl_pin_af!(gpioa, PA7, TIM1, TimerChannel1NPin, AF2);
impl_pin_af!(gpioa, PA11, TIM1, TimerChannel4Pin, AF2);
impl_pin_af!(gpioa, PA10, TIM1, TimerChannel3Pin, AF2);
impl_pin_af!(gpioa, PA12, TIM1, TimerEtrPin, AF2);
impl_pin_af!(gpioa, PA13, TIM1, TimerChannel2Pin, AF13);
impl_pin_af!(gpiob, PB0, TIM1, TimerChannel2NPin, AF2);
impl_pin_af!(gpiob, PB3, TIM1, TimerChannel2Pin, AF1);
impl_pin_af!(gpiob, PB1, TIM1, TimerChannel3NPin, AF2);
impl_pin_af!(gpiob, PB6, TIM1, TimerChannel3Pin, AF1);

// impl_pin_af!(gpioa, PA2, TIM3, TimerChannel1Pin, AF13);
// impl_pin_af!(gpioa, PA4, TIM3, TimerChannel3Pin, AF13);
// impl_pin_af!(gpioa, PA5, TIM3, TimerChannel2Pin, AF13);
// impl_pin_af!(gpioa, PA6, TIM3, TimerChannel2Pin, AF1);
// impl_pin_af!(gpioa, PA7, TIM3, TimerChannel2Pin, AF1);
// impl_pin_af!(gpiob, PB0, TIM3, TimerChannel3Pin, AF1);
// impl_pin_af!(gpiob, PB1, TIM3, TimerChannel4Pin, AF1);
// impl_pin_af!(gpiob, PB4, TIM3, TimerChannel1Pin, AF1);
// impl_pin_af!(gpiob, PB5, TIM3, TimerChannel2Pin, AF1);
// impl_pin_af!(gpiof, PF3, TIM3, TimerChannel3Pin, AF13);

// impl_pin_af!(gpioa, PA4, TIM14, TimerChannel1Pin, AF4);
// impl_pin_af!(gpioa, PA7, TIM14, TimerChannel1Pin, AF4);
// impl_pin_af!(gpiob, PB1, TIM14, TimerChannel1Pin, AF0);
// impl_pin_af!(gpiof, PF0_OSC_IN, TIM14, TimerChannel1Pin, AF2);
// impl_pin_af!(gpiof, PF1_OSC_OUT, TIM14, TimerChannel1Pin, AF13);

// impl_pin_af!(gpioa, PA6, TIM16, TimerChannel1Pin, AF5);
// impl_pin_af!(gpiob, PB5, TIM16, TimerBkInPin, AF2);
// impl_pin_af!(gpiob, PB6, TIM16, TimerChannel1NPin, AF2);
// impl_pin_af!(gpiob, PB8, TIM16, TimerChannel1Pin, AF2);

// impl_pin_af!(gpioa, PA7, TIM17, TimerChannel1Pin, AF5);
// impl_pin_af!(gpioa, PA10, TIM17, TimerBkInPin, AF5);
// impl_pin_af!(gpiob, PB4, TIM17, TimerBkInPin, AF5);
// impl_pin_af!(gpiob, PB7, TIM17, TimerChannel1NPin, AF2);
// impl_pin_af!(gpiob, PB8, TIM17, TimerChannel1Pin, AF13);

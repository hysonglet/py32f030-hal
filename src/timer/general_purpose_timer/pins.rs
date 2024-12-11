use super::{TimerChannel1Pin, TimerChannel2Pin, TimerChannel3Pin, TimerChannel4Pin};
use crate::gpio::{self, gpioa, gpiob, gpiof};
use crate::mcu::peripherals;

impl_pin_af!(gpioa, PA2, TIM3, TimerChannel1Pin, AF13);
impl_pin_af!(gpioa, PA4, TIM3, TimerChannel3Pin, AF13);
impl_pin_af!(gpioa, PA5, TIM3, TimerChannel2Pin, AF13);
impl_pin_af!(gpioa, PA6, TIM3, TimerChannel1Pin, AF1);
impl_pin_af!(gpioa, PA7, TIM3, TimerChannel2Pin, AF1);
impl_pin_af!(gpiob, PB0, TIM3, TimerChannel4Pin, AF1);
impl_pin_af!(gpiob, PB1, TIM3, TimerChannel4Pin, AF1);
impl_pin_af!(gpiob, PB4, TIM3, TimerChannel2Pin, AF1);
impl_pin_af!(gpiob, PB5, TIM3, TimerChannel2Pin, AF1);
impl_pin_af!(gpiof, PF3, TIM3, TimerChannel3Pin, AF13);

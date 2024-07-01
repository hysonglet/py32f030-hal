use super::{CtsPin, RtsPin, RxPin, TxPin};
use crate::gpio::{self, gpioa, gpiob, gpiof};
use crate::macro_def::impl_pin_af;
use crate::mcu::peripherals;

// 指定 引脚功能，生成与外设功能绑定的引脚 trait
impl_pin_af!(gpioa, PA0, USART1, CtsPin, AF1);
impl_pin_af!(gpioa, PA0, USART2, CtsPin, AF4);
impl_pin_af!(gpioa, PA0, USART2, TxPin, AF9);

impl_pin_af!(gpioa, PA1, USART1, RtsPin, AF1);
impl_pin_af!(gpioa, PA1, USART2, RxPin, AF9);

impl_pin_af!(gpioa, PA2, USART1, TxPin, AF1);
impl_pin_af!(gpioa, PA2, USART2, TxPin, AF4);

impl_pin_af!(gpioa, PA3, USART1, RxPin, AF1);
impl_pin_af!(gpioa, PA3, USART2, RxPin, AF4);

impl_pin_af!(gpioa, PA4, USART2, TxPin, AF9);

impl_pin_af!(gpioa, PA5, USART2, TxPin, AF9);

impl_pin_af!(gpioa, PA7, USART1, TxPin, AF8);
impl_pin_af!(gpioa, PA7, USART2, TxPin, AF9);

impl_pin_af!(gpioa, PA8, USART1, TxPin, AF8);
impl_pin_af!(gpioa, PA8, USART2, TxPin, AF9);

impl_pin_af!(gpioa, PA9, USART1, TxPin, AF1);
impl_pin_af!(gpioa, PA9, USART1, RxPin, AF8);
impl_pin_af!(gpioa, PA9, USART2, TxPin, AF4);

impl_pin_af!(gpioa, PA10, USART1, RxPin, AF1);
impl_pin_af!(gpioa, PA10, USART1, TxPin, AF8);
impl_pin_af!(gpioa, PA10, USART2, RxPin, AF4);

impl_pin_af!(gpioa, PA11, USART1, CtsPin, AF1);
impl_pin_af!(gpioa, PA11, USART2, CtsPin, AF4);

impl_pin_af!(gpioa, PA12, USART1, RtsPin, AF1);
impl_pin_af!(gpioa, PA12, USART2, RtsPin, AF4);

impl_pin_af!(gpioa, PA13, USART1, RxPin, AF8);

impl_pin_af!(gpioa, PA14, USART1, TxPin, AF1);
impl_pin_af!(gpioa, PA14, USART2, TxPin, AF4);

impl_pin_af!(gpioa, PA15, USART1, RxPin, AF1);
impl_pin_af!(gpioa, PA15, USART2, RxPin, AF4);

impl_pin_af!(gpiob, PB2, USART1, RxPin, AF0);
impl_pin_af!(gpiob, PB2, USART2, RxPin, AF3);

impl_pin_af!(gpiob, PB3, USART1, RtsPin, AF3);
impl_pin_af!(gpiob, PB3, USART2, RtsPin, AF4);

impl_pin_af!(gpiob, PB5, USART1, CtsPin, AF4);
impl_pin_af!(gpiob, PB4, USART2, CtsPin, AF5);

impl_pin_af!(gpiob, PB6, USART1, TxPin, AF0);
impl_pin_af!(gpiob, PB6, USART2, TxPin, AF4);

impl_pin_af!(gpiob, PB7, USART1, RxPin, AF0);
impl_pin_af!(gpiob, PB7, USART2, RxPin, AF4);

impl_pin_af!(gpiob, PB8, USART1, TxPin, AF8);
impl_pin_af!(gpiob, PB8, USART2, TxPin, AF4);

impl_pin_af!(gpiof, PF0_OSC_IN, USART2, RxPin, AF4);
impl_pin_af!(gpiof, PF0_OSC_IN, USART1, RxPin, AF8);
impl_pin_af!(gpiof, PF0_OSC_IN, USART2, TxPin, AF9);

impl_pin_af!(gpiof, PF1_OSC_OUT, USART2, TxPin, AF4);
impl_pin_af!(gpiof, PF1_OSC_OUT, USART1, TxPin, AF8);
impl_pin_af!(gpiof, PF1_OSC_OUT, USART2, RxPin, AF9);

impl_pin_af!(gpiof, PF2_NRST, USART2, RxPin, AF4);

impl_pin_af!(gpiof, PF3, USART1, TxPin, AF0);
impl_pin_af!(gpiof, PF3, USART2, TxPin, AF4);

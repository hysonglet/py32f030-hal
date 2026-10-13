use super::{MisoPin, MosiPin, NssPin, SckPin};
use crate::gpio::{self, gpioa, gpiob, gpiof};
use crate::mcu::peripherals;

// Assign pin functions and generate the pin traits bound to peripheral functions.
// Implement the corresponding function trait for every pin that has an SPI function.
impl_pin_af!(gpioa, PA0, SPI2, SckPin, AF0);
impl_pin_af!(gpioa, PA0, SPI1, MisoPin, AF10);

impl_pin_af!(gpioa, PA1, SPI1, SckPin, AF0);
impl_pin_af!(gpioa, PA1, SPI1, MosiPin, AF10);

impl_pin_af!(gpioa, PA2, SPI1, MosiPin, AF0);
impl_pin_af!(gpioa, PA2, SPI1, SckPin, AF10);

impl_pin_af!(gpioa, PA3, SPI1, MosiPin, AF0);
impl_pin_af!(gpioa, PA3, SPI1, SckPin, AF10);

impl_pin_af!(gpioa, PA4, SPI1, NssPin, AF0);
impl_pin_af!(gpioa, PA4, SPI2, MosiPin, AF2);

impl_pin_af!(gpioa, PA5, SPI1, SckPin, AF0);

impl_pin_af!(gpioa, PA6, SPI1, MisoPin, AF0);

impl_pin_af!(gpioa, PA7, SPI1, MosiPin, AF0);
impl_pin_af!(gpioa, PA7, SPI1, MisoPin, AF10);

impl_pin_af!(gpioa, PA8, SPI2, NssPin, AF0);
impl_pin_af!(gpioa, PA8, SPI1, MosiPin, AF10);

impl_pin_af!(gpioa, PA9, SPI2, MisoPin, AF0);
impl_pin_af!(gpioa, PA9, SPI1, NssPin, AF10);

impl_pin_af!(gpioa, PA10, SPI2, MosiPin, AF0);
impl_pin_af!(gpioa, PA10, SPI1, NssPin, AF10);

impl_pin_af!(gpioa, PA11, SPI1, MisoPin, AF0);

impl_pin_af!(gpioa, PA12, SPI1, MosiPin, AF0);

impl_pin_af!(gpioa, PA13, SPI1, MisoPin, AF10);

impl_pin_af!(gpioa, PA15, SPI1, NssPin, AF0);

impl_pin_af!(gpiob, PB0, SPI1, NssPin, AF0);
impl_pin_af!(gpiob, PB2, SPI2, SckPin, AF1);

impl_pin_af!(gpiob, PB3, SPI1, SckPin, AF0);
impl_pin_af!(gpiob, PB4, SPI1, MisoPin, AF0);
impl_pin_af!(gpiob, PB5, SPI1, MosiPin, AF0);

impl_pin_af!(gpiob, PB6, SPI2, MisoPin, AF3);
impl_pin_af!(gpiob, PB7, SPI2, MosiPin, AF0);
impl_pin_af!(gpiob, PB8, SPI2, SckPin, AF1);
impl_pin_af!(gpiob, PB8, SPI2, NssPin, AF11);

impl_pin_af!(gpiof, PF0_OSC_IN, SPI2, SckPin, AF3);
impl_pin_af!(gpiof, PF1_OSC_OUT, SPI2, MisoPin, AF3);
impl_pin_af!(gpiof, PF1_OSC_OUT, SPI1, NssPin, AF10);
impl_pin_af!(gpiof, PF2_NRST, SPI2, MosiPin, AF0);
impl_pin_af!(gpiof, PF3, SPI2, MisoPin, AF3);

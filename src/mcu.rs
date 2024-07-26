// 单片机所有抽象的外设资源入口
embassy_hal_internal::peripherals! {
    /// GPIO
    GPIOA,
    GPIOB,
    GPIOF,

    /// USART
    USART1,
    USART2,

    /// I2c
    I2C,

    /// Advanced Timer
    TIM1,

    /// General purpose timer
    TIM3,
    TIM14,
    TIM16,
    TIM17,

    ///Low power timer (LPTIM)
    LPTIM,

    RTC,

    ADC,

    SPI1,
    SPI2,

    IWdg,
    WWdg,

    CRC,

    COMP1,
    COMP2,

    /// DMA
    DmaChannel1,
    DmaChannel2,
    DmaChannel3,
}

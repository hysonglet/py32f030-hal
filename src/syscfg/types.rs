pub enum DmaChannelMap {
    ADC = 0,
    SPI1_TX = 1,
    SPI1_RX = 2,
    SPI2_TX = 3,
    SPI2_RX = 4,
    USART1_TX = 5,
    USART1_RX = 6,
    USART2_TX = 7,
    USART2_RX = 8,

    I2C_TX = 9,
    I2C_RX = 10,

    TIM1_CH1 = 11,
    TIM1_CH2 = 12,
    TIM1_CH3 = 13,
    TIM1_CH4 = 14,
    TIM1_COM = 15,
    TIM1_UP = 16,
    TIM1_TRIG = 17,
    TIM3_CH1 = 18,
    TIM3_CH3 = 19,
    TIM3_CH4 = 20,
    TIM3_TRG = 21,
    TIM3_UP = 22,

    TIM16_CH1 = 24,
    TIM16_UP = 27,
    TIM17_CH1 = 28,
    TIM17_UP = 29,
}

/// 系统启动引导模式
pub enum BootMode {
    /// 主flash启动
    MainFlash = 0,
    /// 进入iap
    SystemFlash = 1,
    /// 从sram启动
    SRAM = 3,
}

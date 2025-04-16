use super::*;
use crate::gpio::{self, AnyPin};
use core::marker::PhantomData;
use embassy_hal_internal::{into_ref, PeripheralRef};

/// PWM
///
pub struct Pwm<'d, T: Instance> {
    _t: PhantomData<&'d T>,

    _channel_1_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_1_n_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_2_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_2_n_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_3_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_3_n_pin: Option<PeripheralRef<'d, AnyPin>>,
    _channel_4_pin: Option<PeripheralRef<'d, AnyPin>>,
}

impl<'d, T: Instance> Default for Pwm<'d, T> {
    fn default() -> Self {
        Self {
            _t: PhantomData,
            _channel_1_pin: None,
            _channel_1_n_pin: None,
            _channel_2_pin: None,
            _channel_2_n_pin: None,
            _channel_3_pin: None,
            _channel_3_n_pin: None,
            _channel_4_pin: None,
        }
    }
}

// pub struct PwmChannel<'d, T: Instance> {
//     _id: Channel,
//     _t: PhantomData<&'d T>,

//     ch_pin: Option<PeripheralRef<'d, AnyPin>>,
// }

#[derive(Default)]
pub struct ChannelOutputConfig {
    pub state: bool,
    pub polarity: bool,
    pub idle_state: bool,
}

pub struct ChannelConfig {
    pub mode: ChannelMode,
    pub clear: bool,
    pub fast: bool,
    pub preload: bool,
    /// Specifies the TIM Output Compare state.
    pub compare: u16,

    pub ch: Option<ChannelOutputConfig>,
    pub n_ch: Option<ChannelOutputConfig>,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            mode: ChannelMode::PWM1,
            clear: false,
            fast: false,
            preload: false,
            compare: 0,
            ch: None,
            n_ch: None,
        }
    }
}

impl ChannelConfig {
    pub fn mode(self, mode: ChannelMode) -> Self {
        Self { mode, ..self }
    }

    pub fn compare(self, compare: u16) -> Self {
        Self { compare, ..self }
    }

    pub fn ch(self, ch: ChannelOutputConfig) -> Self {
        Self {
            ch: Some(ch),
            ..self
        }
    }

    pub fn n_ch(self, n_ch: ChannelOutputConfig) -> Self {
        Self {
            n_ch: Some(n_ch),
            ..self
        }
    }
}

impl<'d, T: Instance> Pwm<'d, T> {
    // 配置函数，用于配置四个通道的参数
    pub fn config(
        &mut self,
        channel_1_config: Option<ChannelConfig>,
        channel_2_config: Option<ChannelConfig>,
        channel_3_config: Option<ChannelConfig>,
        channel_4_config: Option<ChannelConfig>,
    ) -> Result<(), Error> {
        if let Some(config) = channel_1_config {
            Self::channel_config(Channel::CH1, config)?
        }

        if let Some(config) = channel_2_config {
            Self::channel_config(Channel::CH2, config)?
        }

        if let Some(config) = channel_3_config {
            Self::channel_config(Channel::CH3, config)?
        }

        if let Some(config) = channel_4_config {
            Self::channel_config(Channel::CH4, config)?
        }

        T::enable_auto_reload_buff(true);

        Ok(())
    }

    fn channel_config(channel: Channel, config: ChannelConfig) -> Result<(), Error> {
        /* 14, 16, 17只有一个通道 */
        if T::id().is_general_14() || T::id().is_general_16_17() {
            if channel != Channel::CH1 {
                return Err(Error::InvalidChannel);
            }
            // only for edge align mode
        }

        T::set_enable_channel(channel, ChannelOutput::P, false);
        T::set_enable_channel(channel, ChannelOutput::N, false);
        T::set_channel_output_config(
            channel,
            config.mode,
            config.clear,
            config.fast,
            config.preload,
        );
        T::set_channel_type(channel, ChannelType::Pwm);

        if let Some(ch) = config.ch {
            T::set_channel_output_effective_level(
                channel,
                ChannelOutput::P,
                ch.polarity,
                ch.state,
                ch.idle_state,
            );
        }

        if let Some(ch) = config.n_ch {
            T::set_channel_output_effective_level(
                channel,
                ChannelOutput::N,
                ch.polarity,
                ch.state,
                ch.idle_state,
            );
        }

        Ok(())
    }

    pub fn new() -> Result<Self, Error> {
        Ok(Default::default())
    }
}

impl<'d, T: Instance> Pwm<'d, T> {
    pub fn set_channel_1_pin<OC_PIN, OC_N_PIN>(
        &mut self,
        oc_pin: Option<OC_PIN>,
        oc_n_pin: Option<OC_N_PIN>,
    ) -> Result<(), Error>
    where
        OC_PIN: TimerChannel1Pin<T> + 'd,
        OC_N_PIN: TimerChannel1NPin<T>,
    {
        let oc_pin = oc_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        let oc_n_pin = oc_n_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        self._channel_1_pin = oc_pin;
        self._channel_1_n_pin = oc_n_pin;

        Ok(())
    }

    pub fn set_channel_2_pin<OC_PIN, OC_N_PIN>(
        &mut self,
        oc_pin: Option<OC_PIN>,
        oc_n_pin: Option<OC_N_PIN>,
    ) -> Result<(), Error>
    where
        OC_PIN: TimerChannel2Pin<T> + 'd,
        OC_N_PIN: TimerChannel2NPin<T> + 'd,
    {
        /* 14, 16, 17只有一个通道 */
        if T::id().is_general_14() || T::id().is_general_16_17() {
            return Err(Error::InvalidChannel);

            // only for edge align mode
        }
        let oc_pin = oc_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        let oc_n_pin = oc_n_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        self._channel_2_pin = oc_pin;
        self._channel_2_n_pin = oc_n_pin;

        Ok(())
    }

    pub fn set_channel_3_pin<OC_PIN, OC_N_PIN>(
        &mut self,
        oc_pin: Option<OC_PIN>,
        oc_n_pin: Option<OC_N_PIN>,
    ) -> Result<(), Error>
    where
        OC_PIN: TimerChannel3Pin<T> + 'd,
        OC_N_PIN: TimerChannel3NPin<T> + 'd,
    {
        /* 14, 16, 17只有一个通道 */
        if T::id().is_general_14() || T::id().is_general_16_17() {
            return Err(Error::InvalidChannel);
            // only for edge align mode
        }
        let oc_pin = oc_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        let oc_n_pin = oc_n_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        self._channel_3_pin = oc_pin;
        self._channel_3_n_pin = oc_n_pin;

        Ok(())
    }

    pub fn set_channel_4_pin<OC_PIN, OC_N_PIN>(
        &mut self,
        oc_pin: Option<OC_PIN>,
    ) -> Result<(), Error>
    where
        OC_PIN: TimerChannel4Pin<T> + 'd,
    {
        /* 14, 16, 17只有一个通道 */
        if T::id().is_general_14() || T::id().is_general_16_17() {
            return Err(Error::InvalidChannel);

            // only for edge align mode
        }

        let oc_pin = oc_pin.map_or_else(
            || None,
            |pin| {
                into_ref!(pin);
                pin.set_instance_af(gpio::Speed::VeryHigh, gpio::PinIoType::PullUp);
                Some(pin.map_into())
            },
        );

        self._channel_4_pin = oc_pin;

        Ok(())
    }
}

impl<'d, T: Instance> Pwm<'d, T> {
    pub fn enable(&mut self, channel: Channel) {
        T::set_enable_channel(channel, ChannelOutput::P, true);
        T::set_enable_channel(channel, ChannelOutput::N, true);
    }

    pub fn disable(&mut self, channel: Channel) {
        T::set_enable_channel(channel, ChannelOutput::P, false);
        T::set_enable_channel(channel, ChannelOutput::N, false);
    }

    pub fn get_duty(&self, channel: Channel) -> u16 {
        T::get_channel_capture(channel)
    }

    pub fn get_max_duty(&self) -> u16 {
        T::get_reload()
    }

    /// 设置计数频率（该频率为计数器的频率，并非波形的频率）
    pub fn set_frequency(&mut self, freq: fugit::HertzU32) -> Result<(), Error> {
        let freq = freq.to_Hz();

        if T::get_time_pclk() % freq != 0 {
            // defmt::info!(
            //     "The frequency({}) is not a multiple of pclk({})",
            //     freq,
            //     T::get_time_pclk()
            // );
            return Err(Error::Frequency);
        }

        let pre = T::get_time_pclk() / freq;
        T::set_prescaler(pre as u16 - 1);
        Ok(())
    }

    pub fn get_frequency(&self) -> fugit::HertzU32 {
        fugit::HertzU32::from_raw(T::get_time_pclk() / (T::get_prescaler() + 1) as u32)
    }

    fn get_reload(&self) -> u16 {
        T::get_reload()
    }

    fn set_duty(&mut self, channel: Channel, duty: u16) {
        let duty = if duty > self.get_max_duty() {
            self.get_max_duty()
        } else {
            duty
        };
        T::set_channel_compare(channel, duty);
    }

    fn set_auto_reload(&mut self, period: u16) {
        T::set_auto_reload(period)
    }

    pub fn start(&mut self) {
        T::enable_channel_output(true);
        T::start();
    }

    pub fn stop(&mut self) {
        T::enable_channel_output(false);
        T::stop()
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

impl<'d, T: Instance> embedded_hal_027::Pwm for Pwm<'d, T> {
    type Channel = Channel;
    type Duty = u16;
    type Time = fugit::MillisDurationU64;

    fn enable(&mut self, channel: Self::Channel) {
        self.enable(channel)
    }

    fn disable(&mut self, channel: Self::Channel) {
        self.disable(channel)
    }

    fn get_duty(&self, channel: Self::Channel) -> Self::Duty {
        self.get_duty(channel)
    }

    fn get_max_duty(&self) -> Self::Duty {
        self.get_max_duty()
    }

    fn get_period(&self) -> Self::Time {
        let freq = self.get_frequency();
        let period =
            fugit::MillisDurationU64::from_ticks(freq.to_Hz() as u64 / self.get_reload() as u64);
        period
    }

    fn set_duty(&mut self, channel: Self::Channel, duty: Self::Duty) {
        self.set_duty(channel, duty)
    }

    fn set_period<P>(&mut self, period: P)
    where
        P: Into<Self::Time>,
    {
        let auto_reload = period.into().ticks() as u64 / self.get_frequency().to_Hz() as u64;
        if auto_reload > u16::MAX as u64 {
            panic!("The period is too long");
        }
        self.set_auto_reload(auto_reload as u16);
    }
}

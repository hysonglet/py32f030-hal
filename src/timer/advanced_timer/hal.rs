pub(crate) mod sealed {
    use super::super::*;
    use crate::pac;

    pub trait Instance {
        // 考虑以后其他单片机可能有多个IIC
        fn id() -> AdvancedTimer;

        #[inline]
        fn block() -> &'static pac::tim1::RegisterBlock {
            match Self::id() {
                AdvancedTimer::TIM1 => unsafe { pac::TIM1::PTR.as_ref().unwrap() },
            }
        }

        fn base_config(config: BaseConfig) -> Result<(), Error> {
            // Self::block()
        }
    }
}

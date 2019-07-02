/// GPIO Direction
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GpioDirection {
    /// Input mode
    In,
    /// Output mode
    Out,
}

/// Logic Levels
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GpioLevel {
    /// Logic low
    Low,

    /// Logic high
    High,
}

pub trait Gpio {
    /// Set logic level
    fn set_level(&self, level: GpioLevel);

    /// Read current logic level
    fn level(&self) -> GpioLevel;

    /// Set diretion mode to `In` or `Out`,
    /// for reading and writing
    fn set_direction(&self, new_mode: GpioDirection);

    /// Helper function for setting low level
    #[inline(always)]
    fn set_low(&self) {
        self.set_level(GpioLevel::Low)
    }

    /// Helper function for setting high level
    #[inline(always)]
    fn set_high(&self) {
        self.set_level(GpioLevel::High)
    }
}

/// Analog Input
pub trait Adc {
    /// Read analog input value
    fn read(&self) -> u32;
}

/// Minimal configuration info for a gpio pin
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GpioConf {
    pub index: usize,
    pub direction: GpioDirection,
}

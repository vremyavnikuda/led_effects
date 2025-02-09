#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

//! LED Effects library for embedded Rust systems
//!
//! This library provides various LED effects for embedded systems using PWM.
//! It is designed to be platform-agnostic and works with any microcontroller
//! that implements the embedded-hal traits.


use core::marker::PhantomData;
// Исправляем импорт для embedded-hal 0.2.7
use embedded_hal::PwmPin;
use cortex_m::asm;

#[cfg(feature = "defmt")]
use defmt::Format;

/// Error type for LED effects
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum Error {
    /// PWM-related error
    Pwm,
    /// Invalid parameter error
    InvalidParameter,
}

/// Main structure for LED effects
pub struct LEDEffect<PWM>
where
    PWM: PwmPin,
{
    pin: PWM,
    pwm_min: PWM::Duty,
    pwm_max: PWM::Duty,
    pwm_mid: PWM::Duty,
    _phantom: PhantomData<PWM>,
}

impl<PWM> LEDEffect<PWM>
where
    PWM: PwmPin,
    PWM::Duty: Into<u32> + From<u32> + Copy + Ord,
{
    /// Create a new LEDEffect instance
    pub fn new(mut pin: PWM, pwm_min: PWM::Duty, pwm_max: PWM::Duty) -> Result<Self, Error> {
        if pwm_max <= pwm_min {
            return Err(Error::InvalidParameter);
        }

        let pwm_mid = From::from(
            pwm_min.into() + (pwm_max.into() - pwm_min.into()) / 2
        );

        pin.enable();

        Ok(Self {
            pin,
            pwm_min,
            pwm_max,
            pwm_mid,
            _phantom: PhantomData,
        })
    }

    /// Create heartbeat effect
    pub fn heartbeat(
        &mut self,
        flash_beats: u32,
        grouped_as: u32,
        bpm: u32
    ) -> Result<(), Error> {
        let period_time = (60_000 / bpm) / 6;
        let short_period_time = period_time / 3;
        let down_delay_time = (period_time * 2) / (self.pwm_mid.into() - self.pwm_min.into());

        for n in 1..=flash_beats {
            self.pin.set_duty(self.pwm_max);
            self.delay_ms(short_period_time);

            self.pin.set_duty(self.pwm_min);
            self.delay_ms(short_period_time * 2);

            self.pin.set_duty(self.pwm_mid);

            let mut current = self.pwm_mid;
            while current >= self.pwm_min {
                self.pin.set_duty(current);
                self.delay_ms(down_delay_time);
                current = From::from(current.into().saturating_sub(1));
            }

            let wait = if n % grouped_as != 0 {
                period_time
            } else if grouped_as == 1 {
                period_time * 2
            } else {
                (period_time * 2) + (grouped_as * period_time)
            };

            self.delay_ms(wait);
        }
        self.pin.set_duty(From::from(0u32));
        Ok(())
    }

    /// Create breathing effect
    pub fn breath(&mut self, duration: u32) -> Result<(), Error> {
        let period_time = duration / 6;
        let up_delay = (period_time * 2) / (self.pwm_max.into() - self.pwm_min.into());
        let down_delay = (period_time * 2) / (self.pwm_max.into() - self.pwm_min.into());

        let mut current = self.pwm_min;
        while current < self.pwm_max {
            self.pin.set_duty(current);
            self.delay_ms(up_delay);
            current = From::from(current.into().saturating_add(1));
        }

        current = self.pwm_max;
        while current > self.pwm_min {
            self.pin.set_duty(current);
            self.delay_ms(down_delay);
            current = From::from(current.into().saturating_sub(1));
        }

        self.delay_ms(period_time * 2);
        self.pin.set_duty(From::from(0u32));
        Ok(())
    }

    /// Destroy the LED effect instance and return the underlying pin
    pub fn destroy(self) -> PWM {
        self.pin
    }

    /// Delays execution for a specified number of milliseconds.
    ///
    /// This function uses a busy-wait loop to delay execution for the given
    /// number of milliseconds. The delay is achieved by converting the given
    /// time into clock cycles and using the `asm::delay` function to wait
    /// for the specified number of cycles.
    ///
    /// # Arguments
    ///
    /// * `ms` - The number of milliseconds to delay execution.
    ///
    /// # Example
    ///
    /// ```
    /// led_effect.delay_ms(500); // Delays for 500 milliseconds
    /// ```
    #[inline(always)]
    fn delay_ms(&self, ms: u32) {
        let cycles = ms * self.clock_cycles_per_ms();
        asm::delay(cycles);
    }

    /// Calculate the number of clock cycles per millisecond.
    ///
    /// This function returns the number of clock cycles that occur in one millisecond
    /// based on the system clock frequency. For example, for a system running at 48MHz,
    /// it returns 48,000 cycles per millisecond. Adjust the returned value if the system
    /// clock frequency changes.
    ///
    /// # Returns
    ///
    /// * `u32` - The number of clock cycles in one millisecond.
    ///```
    ///#[inline(always)]
    ///fn clock_cycles_per_ms(&self) -> u32 {
    ///    // This should be adjusted based on your system clock
    ///    // For example, for a 48MHz system:
    ///    48_000 // cycles per ms at 48MHz
    ///}
    /// ```
    #[inline(always)]
    fn clock_cycles_per_ms(&self) -> u32 {
        // This should be adjusted based on your system clock
        // For example, for a 48MHz system:
        48_000 // cycles per ms at 48MHz
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Создаем мок для тестирования
    struct MockPwm {
        duty: u32,
    }

    impl MockPwm {
        /// Creates a new instance of `MockPwm` with the default duty cycle.
        ///
        /// This function initializes a `MockPwm` instance with a duty cycle set to `0`.
        ///
        /// # Returns
        ///
        /// * `Self` - A new `MockPwm` instance with the duty cycle initialized to `0`.
        ///
        /// ```
        ///fn new() -> Self {
        ///    Self { duty: 0 }
        ///}
        /// ```
        fn new() -> Self {
            Self { duty: 0 }
        }
    }

    impl PwmPin for MockPwm {
        type Duty = u32;

        /// Disables the PWM output.
        ///
        /// This function disables the PWM output and stops updating the duty cycle.
        /// After calling this function, the `duty` field is no longer updated.
        /// ```
        ///#[inline(always)]
        ///fn disable(&mut self) {}
        /// ```
        fn disable(&mut self) {}
        /// Enables the PWM output.
        ///
        /// This function enables the PWM output and starts updating the duty cycle
        /// based on the value of the `duty` field.
        ///
        fn enable(&mut self) {}
        /// Returns the current duty cycle of the PWM pin.
        ///
        /// This function retrieves the current duty cycle value of the PWM pin.
        /// The duty cycle is a measure of the proportion of 'on' time to the
        /// regular interval or 'period' of the PWM signal. A higher duty cycle
        /// corresponds to a brighter LED when used in LED applications.
        ///
        /// # Returns
        ///
        /// * `Self::Duty` - The current duty cycle of the PWM pin.
        ///
        fn get_duty(&self) -> Self::Duty {
            self.duty
        }
        /// Returns the maximum possible duty cycle value of the PWM pin.
        ///
        /// # Returns
        ///
        /// * `Self::Duty` - The maximum possible duty cycle value of the PWM pin.
        ///
        fn get_max_duty(&self) -> Self::Duty {
            255
        }
        /// Sets the duty cycle of the PWM pin.
        ///
        /// This function sets the duty cycle of the PWM pin to the given value.
        /// The duty cycle is a measure of the proportion of 'on' time to the
        /// regular interval or 'period' of the PWM signal. A higher duty cycle
        /// corresponds to a brighter LED when used in LED applications.
        ///
        /// # Arguments
        ///
        /// * `duty` - The new duty cycle value of the PWM pin.
        ///
        fn set_duty(&mut self, duty: Self::Duty) {
            self.duty = duty;
        }
    }

    /// Tests creating a new instance of the `LEDEffect` struct.
    ///
    /// This test creates a new instance of the `LEDEffect` struct with a valid
    /// set of parameters. The test asserts that the `LEDEffect` instance is created
    /// successfully.
    #[test]
    fn test_new_led_effect() {
        let pin = MockPwm::new();
        let led = LEDEffect::new(pin, 5, 255);
        assert!(led.is_ok());
    }

    /// Tests that creating a new `LEDEffect` instance with invalid parameters fails.
    ///
    /// This test creates a new instance of the `LEDEffect` struct with an invalid
    /// set of parameters (i.e. `pwm_min` >= `pwm_max`). The test asserts that the
    /// `LEDEffect` instance cannot be created and that the error variant is
    /// `Error::InvalidParameter`.
    ///
    #[test]
    fn test_invalid_parameters() {
        let pin = MockPwm::new();
        let led = LEDEffect::new(pin, 255, 5);
        assert!(matches!(led, Err(Error::InvalidParameter)));
    }
}
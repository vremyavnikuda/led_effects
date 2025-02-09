#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_probe as _;
use stm32f1xx_hal::{
    pac,
    prelude::*,
    timer::{Timer, Channel},
    pwm::*,
};
use led_effects::LEDEffect;

#[cfg(feature = "defmt")]
use defmt_rtt as _;

/// Entry point of the embedded application.
///
/// This function initializes the peripherals and configures the system clock.
/// It sets up the PWM on pin PA0 using TIM2_CH1 at a frequency of 1 kHz.
/// An LED effect is created and demonstrated using PWM signals, implementing
/// both a breathing and heartbeat effect in an infinite loop. Error handling
/// is performed with optional logging via defmt for debugging purposes.
///#[entry]
///fn main() -> ! {
///    // Function implementation
///}
#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let mut afio = dp.AFIO.constrain();

    let clocks = rcc.cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    let mut gpioa = dp.GPIOA.split();

    // Настройка PWM на PA0 (TIM2_CH1)
    let c1 = gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl);
    let mut pwm = Timer::new(dp.TIM2, &clocks).pwm(
        c1,
        &mut afio.mapr,
        1.kHz(),
    );

    // Получаем канал PWM
    let max_duty = pwm.get_max_duty();
    let mut pwm_ch = pwm.split().0;
    pwm_ch.enable();

    let mut led = LEDEffect::new(pwm_ch, max_duty / 50, max_duty)
        .expect("Failed to create LED effect");

    #[cfg(feature = "defmt")]
    defmt::info!("LED Effects Demo Starting...");

    loop {
        if let Err(_) = led.breath(5454) {
            #[cfg(feature = "defmt")]
            defmt::error!("Breathing effect failed");
            continue;
        }

        cortex_m::asm::delay(48_000_000);

        if let Err(_) = led.heartbeat(2, 1, 60) {
            #[cfg(feature = "defmt")]
            defmt::error!("Heartbeat effect failed");
            continue;
        }

        cortex_m::asm::delay(48_000_000);
    }
}

/// This function handles the Hard Fault exception in the Cortex-M processor.
///
/// # Safety
/// This function is marked as `unsafe` because it deals with low-level
/// exception handling that could lead to undefined behavior if misused.
///
/// This function enters an infinite loop upon encountering a Hard Fault
/// to halt the system, preventing further execution. If the `defmt` feature
/// is enabled, it logs the occurrence of the Hard Fault.
#[cortex_m_rt::exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    #[cfg(feature = "defmt")]
    defmt::error!("Hard Fault");

    loop {}
}
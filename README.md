# LED Effects Library for Embedded Rust

Welcome to the LED Effects Library for Embedded Rust! This library provides various LED effects for embedded systems using PWM. It is designed to be platform-agnostic and works with any microcontroller that implements the embedded-hal traits.
___
## Features
- Breathing effect: Smooth fade in/out
- Heartbeat effect: Simulated heartbeat pattern
- Flicker effect: Random brightness changes
___

## Getting Started
Prerequisites

To use this library, you will need:

- Rust toolchain installed
- A microcontroller that supports PWM and is compatible with embedded-hal
- cargo build system
___
## Installation
Add the following to your `Cargo.toml`:

```toml
[dependencies]
embedded-hal = "0.2.7"
nb = "1.1.0"
defmt = { version = "0.3", optional = true }
critical-section = "1.1"
cortex-m = "0.6.4"
stm32f1xx-hal = { version = "0.10", features = ["stm32f103", "rt"] }

[dev-dependencies]
embedded-hal-mock = "0.9"
cortex-m-rt = "0.7"
panic-probe = { version = "0.3", features = ["print-defmt"] }
defmt-rtt = "0.4"
```
___
## Usage

```rust
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

    // We get the PWM channel
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

#[cortex_m_rt::exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    #[cfg(feature = "defmt")]
    defmt::error!("Hard Fault");

    loop {}
}
```
___
## Building and Running
To build and run the example, follow these steps:

1. Install the required target and tools:
```shell
rustup target add thumbv7m-none-eabi
cargo install cargo-binutils
rustup component add llvm-tools-preview
cargo install probe-run 
```

2. Create the `.cargo/config.toml` file:
```toml
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
rustflags = [
  "-C", "link-arg=-Tlink.x",
]

[build]
target = "thumbv7m-none-eabi"

[env]
DEFMT_LOG = "trace"
```
3. Create the `memory.x` file:
```
MEMORY
{
  /* STM32F103C8 имеет 64K FLASH и 20K RAM */
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K
  RAM : ORIGIN = 0x20000000, LENGTH = 20K
}

/* correctly place sections in memory */
SECTIONS
{
  /* ... */
}
```
4. Compile and run the example:
```bash
cargo run --example stm32f1xx --features defmt --release
```
___
## License
This project is licensed under the MIT License - see the LICENSE file for details.
___
## Contributing
Contributions are welcome! Please open an issue or submit a pull request.
___
## Acknowledgements
Special thanks to the Rust Embedded community for their excellent libraries and support.
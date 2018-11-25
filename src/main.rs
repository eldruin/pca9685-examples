//! Control a servo connected to channel 0 and one connected to channel 1.
//! Make the servo at channel 0 turn clockwise, then counter-clockwise and
//! the servo at channel 1 does the opposite.
#![deny(unsafe_code, warnings)]
#![no_std]
#![no_main]

extern crate cortex_m;
#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;
extern crate f3;
extern crate panic_semihosting;
extern crate pwm_pca9685 as pca9685;

use f3::hal::delay::Delay;
use f3::hal::prelude::*;
use f3::hal::stm32f30x;
use f3::led::Led;
use rt::ExceptionFrame;
use f3::hal::i2c::I2c;
pub use f3::hal::stm32f30x::i2c1;
use pca9685::{Channel, Pca9685, SlaveAddr};

entry!(main);

fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut led: Led = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper)
        .into();
    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);

    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 100.khz(), clocks, &mut rcc.apb1);
    let mut pwm = Pca9685::new(i2c, SlaveAddr::default());
    // This results in about 60 Hz, which is the frequency at which servos operate.
    pwm.set_prescale(100).unwrap();
    pwm.enable().unwrap();
    // Turn all channels on at time "0".
    pwm.set_channel_on(Channel::All, 0).unwrap();

    // You need to tweak these min/max values for your servos as these may vary.
    let servo_min = 130; // minimum pulse length (out of 4096)
    let servo_max = 610; // maximum pulse length (out of 4096)
    let mut current = servo_min;
    let mut factor: i16 = 1;
    loop {
        // Blink LED 0 (really fast, it will seem to be on the whole time)
        // to check that everything is actually running.
        // If the LED 0 is off, something went wrong.
        led.on();
        delay.delay_ms(1_u16);
        led.off();
        delay.delay_ms(1_u16);

        pwm.set_channel_off(Channel::C0, current).unwrap();
        pwm.set_channel_off(Channel::C1, servo_min + (servo_max - current)).unwrap();

        if current == servo_max {
            factor = -1;
        }
        else if current == servo_min {
            factor = 1;
        }
        current = (current as i16 + factor) as u16;
    }
}

exception!(HardFault, hard_fault);

fn hard_fault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}

exception!(*, default_handler);

fn default_handler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}

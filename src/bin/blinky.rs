#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::{bind_interrupts, peripherals, usart};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut led_green = Output::new(p.PG13, Level::Low, Speed::Low);
    let mut led_red = Output::new(p.PG14, Level::Low, Speed::Low);

    loop {
        info!("Green");
        led_green.set_high();
        led_red.set_low();
        Timer::after_millis(1000).await;

        info!("Red");
        led_green.set_low();
        led_red.set_high();
        Timer::after_millis(1000).await;
    }
}

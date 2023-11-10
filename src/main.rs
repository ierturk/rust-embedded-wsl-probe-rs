#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::cell::RefCell;
use defmt::*;
use display::Display;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::time::Hertz;
use embassy_stm32::{gpio, peripherals, Config};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, ThreadModeRawMutex};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use gpio::{Input, Level, Output, Pull, Speed};
use {defmt_rtt as _, panic_probe as _};

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};

mod display;

type LedOutput = Output<'static, peripherals::PG13>;
type ButtonInput = ExtiInput<'static, peripherals::PA0>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Program start");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL168,
            divp: Some(Pllp::DIV2), // 8mhz / 4 * 168 / 2 = 168Mhz.
            divq: Some(Pllq::DIV7), // 8mhz / 4 * 168 / 7 = 48Mhz.
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }
    let p = embassy_stm32::init(config);

    // let p = embassy_stm32::init(Default::default());
    let led: LedOutput = Output::new(p.PG13, Level::Low, Speed::Low);

    let button = Input::new(p.PA0, Pull::None);
    let button: ButtonInput = ExtiInput::new(button, p.EXTI0);

    let display = display::init(p.PF9, p.PF7, p.PC2, p.PG14, p.PD13, p.SPI5);

    unwrap!(spawner.spawn(blinker(led, Duration::from_millis(500))));
    unwrap!(spawner.spawn(button_monitor(button)));
    unwrap!(spawner.spawn(display_refresh(display)));
}

/// Blink the physical LED, and a matching indicator on the LCD display
#[embassy_executor::task]
async fn blinker(mut led: LedOutput, interval: Duration) {
    let mut blink = false;
    loop {
        led.set_level(if blink { Level::Low } else { Level::High });
        display_state_update(|s| s.indicator1 = blink);
        blink = !blink;
        Timer::after(interval).await;
    }
}

/// Monitor the button, and show an indicator on the LCD display
#[embassy_executor::task]
async fn button_monitor(mut button: ButtonInput) {
    loop {
        button.wait_for_any_edge().await;
        let level = button.get_level();
        display_state_update(|s| s.indicator2 = level == Level::High);
    }
}

#[derive(Clone)]
struct DisplayState {
    indicator1: bool,
    indicator2: bool,
}

static DISPLAY_STATE: Mutex<ThreadModeRawMutex, RefCell<DisplayState>> =
    Mutex::new(RefCell::new(DisplayState {
        indicator1: false,
        indicator2: false,
    }));
static DISPLAY_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

// Keep the display up to date
#[embassy_executor::task]
async fn display_refresh(mut display: Display) {
    render_background(&mut display);
    loop {
        DISPLAY_SIGNAL.wait().await;
        let state = DISPLAY_STATE.lock(|s| s.borrow().clone());
        render_indicator(&mut display, Point::new(120, 120), state.indicator1);
        render_indicator(&mut display, Point::new(180, 120), state.indicator2);
    }
}

fn display_state_update<F>(mut sfn: F)
where
    F: FnMut(&mut DisplayState) -> (),
{
    DISPLAY_STATE.lock(|s| sfn(&mut s.borrow_mut()));
    DISPLAY_SIGNAL.signal(());
}

fn render_background(display: &mut Display) {
    let test_text = "Erturk Me";

    Rectangle::new(Point::new(0, 0), Size::new(320, 240))
        .into_styled(display.styles.black_fill)
        .draw(&mut display.interface)
        .unwrap();

    Text::with_text_style(
        test_text,
        Point::new(60, 0),
        display.styles.char,
        display.styles.text,
    )
    .draw(&mut display.interface)
    .unwrap();
}

/// Draw an "LED" on the LCD display
fn render_indicator(display: &mut Display, centre: Point, state: bool) -> () {
    let led_size: u32 = 30;
    let led_at = Point::new(
        centre.x - (led_size as i32) / 2,
        centre.y - (led_size as i32) / 2,
    );

    if state {
        Circle::new(led_at, led_size)
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(Rgb565::GREEN)
                    .build(),
            )
            .draw(&mut display.interface)
            .unwrap();
    } else {
        Circle::new(led_at, led_size)
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(Rgb565::CSS_DARK_GRAY)
                    .build(),
            )
            .draw(&mut display.interface)
            .unwrap();
    }
}

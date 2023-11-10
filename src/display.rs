use embassy_stm32::{
    gpio::{Level, Output, Speed},
    peripherals::{self},
    spi,
};
use embedded_graphics::{
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    prelude::RgbColor,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder},
    text::{Baseline, TextStyle},
};
use {defmt_rtt as _, panic_probe as _};

use display_interface_spi::SPIInterface;
use embassy_stm32::dma::NoDma;
use embassy_stm32::time::Hertz;
use ili9341::{Ili9341, Orientation};

pub type DisplayInterface = Ili9341<
    SPIInterface<
        embassy_stm32::spi::Spi<'static, peripherals::SPI5, NoDma, NoDma>,
        Output<'static, peripherals::PD13>,
        Output<'static, peripherals::PC2>,
    >,
    Output<'static, peripherals::PG14>,
>;

pub struct Display {
    pub interface: DisplayInterface,
    pub styles: Styles,
}

pub fn init(
    mosi: peripherals::PF9,
    clk: peripherals::PF7,
    cs: peripherals::PC2,
    reset: peripherals::PG14,
    dc: peripherals::PD13,
    spi: peripherals::SPI5,
) -> Display {
    let cs = Output::new(cs, Level::High, Speed::VeryHigh);
    let reset = Output::new(reset, Level::Low, Speed::VeryHigh);
    let dc = Output::new(dc, Level::Low, Speed::VeryHigh);

    let display_spi = {
        let mut config = spi::Config::default();
        config.frequency = Hertz(1_000_000);
        config.mode = ili9341::SPI_MODE;
        spi::Spi::new_txonly(spi, clk, mosi, NoDma, NoDma, config)
    };

    let interface: DisplayInterface = {
        let mut delay = embassy_time::Delay {};
        Ili9341::new(
            SPIInterface::new(display_spi, dc, cs),
            reset,
            &mut delay,
            Orientation::LandscapeFlipped,
            ili9341::DisplaySize240x320,
        )
        .unwrap()
    };

    let styles = Styles::new();

    Display { interface, styles }
}

/// Some shared styles
pub struct Styles {
    pub char: MonoTextStyle<'static, Rgb565>,
    pub text: TextStyle,
    pub black_fill: PrimitiveStyle<Rgb565>,
    pub white_fill: PrimitiveStyle<Rgb565>,
}

impl Styles {
    fn new() -> Styles {
        let char = MonoTextStyle::new(&profont::PROFONT_24_POINT, Rgb565::WHITE);
        let text = TextStyle::with_baseline(Baseline::Top);
        let black_fill = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::BLACK)
            .build();
        let white_fill = PrimitiveStyleBuilder::new()
            .fill_color(Rgb565::WHITE)
            .build();
        Styles {
            char,
            text,
            black_fill,
            white_fill,
        }
    }
}

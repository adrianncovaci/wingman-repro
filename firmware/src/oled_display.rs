use embedded_graphics::geometry::AnchorPoint;
use embedded_graphics::mono_font::ascii::{FONT_6X10, FONT_7X13_BOLD};
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line, PrimitiveStyle};
use embedded_graphics::text::{Alignment, Text};
use embedded_hal::blocking::delay::DelayMs;
use firmware_logic::Timestamp;
use rtic_monotonics::stm32::fugit::{self, RateExtU32};
use ssd1351::builder::Builder;
use ssd1351::interface::SpiInterface;
use ssd1351::mode::GraphicsMode;
use ssd1351::properties::DisplayRotation;
use stm32h7xx_hal::device::SPI1;
use stm32h7xx_hal::gpio::{Output, Pin, Speed};
use stm32h7xx_hal::rcc::rec::Spi1;
use stm32h7xx_hal::rcc::CoreClocks;
use stm32h7xx_hal::spi;
use stm32h7xx_hal::spi::{
    Config as SpiConfig, Enabled, HardwareCS, HardwareCSMode, Polarity, Spi, SpiExt,
};

pub type GraphicsDisplay = GraphicsMode<SpiInterface<Spi<SPI1, Enabled, u8>, Pin<'G', 9, Output>>>;

const INITIALIZING_TEXT: &str = "ENATA Wingman Gen 2\n initializing ...";
const HEADER_TEXT: &str = "ENATA Wingman Gen2";

pub struct OledDisplay {
    graphic_display: GraphicsDisplay,
    step: u64,
    duration: fugit::Duration<u64, 1, 1_000>,
    connected_to_control_server: bool,
}

static mut DISPLAY_BUFFER: [u8; 128 * 128 * 2] = [0; 128 * 128 * 2];

impl OledDisplay {
    #[allow(clippy::too_many_arguments)]
    pub fn new<DELAY>(
        spi1: SPI1,
        peripherals_spi1: Spi1,
        sck: Pin<'G', 11>,
        mosi: Pin<'D', 7>,
        dc: Pin<'G', 9>,
        ncs: Pin<'G', 10>,
        rst: Pin<'J', 15>,
        timer: &mut DELAY,
        clocks: CoreClocks,
    ) -> Self
    where
        DELAY: DelayMs<u8>,
    {
        let sck = sck.into_alternate().speed(Speed::Medium);
        let mosi = mosi.into_alternate().speed(Speed::Medium);
        let dc = dc.into_push_pull_output().speed(Speed::Medium);
        let ncs = ncs.into_alternate::<5>().speed(Speed::Medium);

        // let mut delay = Delay::new(syst, clocks);

        let spi_config = SpiConfig::new(spi::MODE_3).hardware_cs(HardwareCS {
            mode: HardwareCSMode::WordTransaction,
            assertion_delay: 3.0,
            polarity: Polarity::IdleHigh,
        });
        let spi: spi::Spi<_, _, u8> = spi1.spi(
            (sck, spi::NoMiso, mosi, ncs),
            spi_config,
            20.MHz(),
            peripherals_spi1,
            &clocks,
        );

        let mut display: GraphicsMode<_> = unsafe {
            Builder::new()
                .connect_spi(spi, dc, &mut DISPLAY_BUFFER)
                .into()
        };

        let mut rst = rst.into_push_pull_output();
        rst.set_high();

        display
            .reset(&mut rst, timer)
            .expect("Failed to reset display");
        display.init().expect("Failed to initialize the display");
        display
            .set_rotation(DisplayRotation::Rotate0)
            .expect("Failed to set the display rotation");
        display.clear(true);
        let mut this = Self {
            graphic_display: display,
            step: 0,
            connected_to_control_server: false,
            duration: fugit::Duration::<u64, 1, 1000>::from_ticks(0),
        };

        this.display(|graphic_display| {
            rtt_debug!("Display initialized");
            let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
            Self::display_text(
                graphic_display,
                INITIALIZING_TEXT,
                graphic_display.bounding_box().center() + Point::new(0, 15),
                Alignment::Center,
                character_style,
            );
        });

        this
    }

    pub fn update(&mut self) {
        // Display: 128 x 128 pixels
        self.step += 1;

        let steps = self.step;
        let duration = self.duration;
        let network_status = self.connected_to_control_server;
        self.display(|display| {
            display.clear(false);

            let character_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
            Self::display_header(display);
            Self::display_counter(steps, display, character_style);
            Self::display_network_status(network_status, display);
            Self::display_human_readable_time(display, character_style, duration)
        });
    }

    pub fn set_network_status(&mut self, is_connected: bool) {
        self.connected_to_control_server = is_connected;
    }

    pub fn set_timestamp(&mut self, timestamp: Timestamp) {
        self.duration = timestamp.into();
    }

    fn display(&mut self, d: impl FnOnce(&mut GraphicsDisplay)) {
        d(&mut self.graphic_display);

        self.graphic_display.flush();
    }

    fn display_text(
        display: &mut GraphicsDisplay,
        text: &str,
        position: Point,
        alignment: Alignment,
        font: MonoTextStyle<Rgb565>,
    ) {
        Text::with_alignment(text, position, font, alignment)
            .draw(display)
            .unwrap_or_default();
    }

    fn display_counter(
        steps: u64,
        display: &mut GraphicsDisplay,
        character_style: MonoTextStyle<Rgb565>,
    ) {
        let mut buf = [0u8; 64];
        let ticks_text = format_no_std::show(&mut buf, format_args!("steps: {}", steps)).unwrap();
        Self::display_text(
            display,
            ticks_text,
            display.bounding_box().anchor_point(AnchorPoint::BottomLeft),
            Alignment::Left,
            character_style,
        );
    }

    fn display_header(display: &mut GraphicsDisplay) {
        Self::display_text(
            display,
            HEADER_TEXT,
            display.bounding_box().anchor_point(AnchorPoint::TopCenter) + Point::new(0, 8),
            Alignment::Center,
            MonoTextStyle::new(&FONT_7X13_BOLD, Rgb565::RED),
        );
    }

    fn display_network_status(is_connected: bool, display: &mut GraphicsDisplay) {
        if is_connected {
            Self::draw_network_connected_check(display);
        } else {
            Self::draw_network_disconnected_x(display);
        }
    }

    fn display_human_readable_time(
        display: &mut GraphicsDisplay,
        character_style: MonoTextStyle<Rgb565>,
        duration: fugit::Duration<u64, 1, 1_000>,
    ) {
        let mut buf = [0u8; 64];
        let seconds = duration.to_secs() % 60;
        let minutes = duration.to_minutes() % 60;
        let hours = duration.to_hours();
        let ticks_text = format_no_std::show(
            &mut buf,
            format_args!("{:02}:{:02}:{:02}", hours, minutes, seconds),
        )
        .unwrap();
        Self::display_text(
            display,
            ticks_text,
            display
                .bounding_box()
                .anchor_point(AnchorPoint::BottomRight),
            Alignment::Right,
            character_style,
        );
    }

    fn draw_network_connected_check(display: &mut GraphicsDisplay) {
        Line::new(Point::new(10, 52), Point::new(20, 62))
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_SEA_GREEN, 2))
            .draw(display)
            .unwrap_or_default();

        Line::new(Point::new(20, 62), Point::new(40, 32))
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::CSS_SEA_GREEN, 2))
            .draw(display)
            .unwrap_or_default();
    }

    fn draw_network_disconnected_x(display: &mut GraphicsDisplay) {
        Line::new(Point::new(10, 32), Point::new(40, 62))
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 2))
            .draw(display)
            .unwrap_or_default();

        Line::new(Point::new(10, 62), Point::new(40, 32))
            .into_styled(PrimitiveStyle::with_stroke(Rgb565::RED, 2))
            .draw(display)
            .unwrap_or_default();
    }
}

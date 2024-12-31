use display_interface_spi::SPIInterface;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::FONT_9X18;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::primitives::{Circle, Primitive, PrimitiveStyle, Triangle};
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio;
use esp_idf_svc::hal::gpio::AnyIOPin;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::spi::config::{Config, DriverConfig};
use esp_idf_svc::hal::spi::{Dma, SpiBusDriver, SpiDriver};
use esp_idf_svc::hal::units::MegaHertz;
use mipidsi::models::ILI9341Rgb565;
use mipidsi::Builder;
use std::thread;
use std::time::Duration;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Starting...");

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // Reset, -1 or 4
    let mut rst = gpio::PinDriver::output(pins.gpio4).unwrap();
    rst.set_high().expect("Cannot set reset high");
    // Data Command control pin
    let dc = gpio::PinDriver::output(pins.gpio2).unwrap();

    // Espressif built-in delay provider for small delays
    let mut delay = Ets;

    // Pin 14, Serial Clock
    let sclk = pins.gpio14;
    // Pin 13, MOSI/SDO, Master Out Slave In
    let mosi = pins.gpio13;
    // Pin 12, MISO/SDI, Master In Slave Out
    let _miso = pins.gpio12;
    let cs = gpio::PinDriver::output(pins.gpio15).unwrap();

    let spi = SpiDriver::new(
        peripherals.spi2,
        sclk,
        mosi,
        None::<AnyIOPin>,
        &DriverConfig::new().dma(Dma::Disabled),
    )
    .unwrap();

    // Create the SPI bus driver first
    let spi_bus = SpiBusDriver::new(spi, &Config::new().baudrate(MegaHertz(40).into())).unwrap();

    // Then create the device
    let spi_device = ExclusiveDevice::new_no_delay(spi_bus, cs).unwrap();
    let di = SPIInterface::new(spi_device, dc);
    let mut display = Builder::new(ILI9341Rgb565, di)
        .reset_pin(rst)
        .init(&mut delay)
        .unwrap();

    let mut bl = gpio::PinDriver::output(pins.gpio21).unwrap();
    // Turn on backlight
    bl.set_high().expect("Cannot set backlight high");

    // Force the GPIO to hold it's high state
    core::mem::forget(bl);

    display.clear(Rgb565::BLACK).unwrap();

    // Create text style
    let mut style = MonoTextStyle::new(&FONT_9X18, Rgb565::WHITE);

    // Position x:5, y: 10
    Text::new("Hello", Point::new(5, 10), style)
        .draw(&mut display)
        .unwrap();

    // Turn text to blue
    style.set_text_color(Some(Rgb565::BLUE));
    Text::new("World", Point::new(160, 26), style)
        .draw(&mut display)
        .unwrap();

    // Draw a smiley face with white eyes and a red mouth
    draw_smiley(&mut display).unwrap();

    log::info!("The end.");
    loop {
        thread::sleep(Duration::from_millis(1000));
    }
}

fn draw_smiley<T: DrawTarget<Color = Rgb565>>(display: &mut T) -> Result<(), T::Error> {
    // Draw the left eye as a circle located at (50, 100), with a diameter of 40, filled with white
    Circle::new(Point::new(50, 100), 40)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(display)?;

    // Draw the right eye as a circle located at (140, 100), with a diameter of 40, filled with white
    Circle::new(Point::new(140, 100), 40)
        .into_styled(PrimitiveStyle::with_fill(Rgb565::WHITE))
        .draw(display)?;

    // Draw an upside down red triangle to represent a smiling mouth
    Triangle::new(
        Point::new(90, 150),
        Point::new(140, 150),
        Point::new(115, 180),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
    .draw(display)?;

    // Cover the top part of the mouth with a black triangle so it looks closed instead of open
    Triangle::new(
        Point::new(90, 140),
        Point::new(140, 140),
        Point::new(115, 170),
    )
    .into_styled(PrimitiveStyle::with_fill(Rgb565::BLACK))
    .draw(display)?;

    Ok(())
}

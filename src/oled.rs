use anyhow::Result;

use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};

use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{Output, Pin, PinDriver},
};

use ssd1306::{
    mode::BufferedGraphicsMode,
    prelude::*,
    size::DisplaySize128x64,
    Ssd1306,
};

use crate::exceptions::OledErorr;


/// Oled screen driver
pub struct OledDisplay<DI> {
    display: Ssd1306<DI, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>,
    text_style: MonoTextStyle<'static, BinaryColor>,
}

impl<DI> OledDisplay<DI>
where
    DI: WriteOnlyDataCommand,
{
    /// Create new screen instance and enable it
    ///
    /// # Arguments
    /// `interface` - Configured SPI
    /// `rst_pin` - Reset screen pin driver
    ///
    /// # Returns
    /// `screen` - Screen driver
    /// `text_style` - Text style
    pub fn new<P: Pin>(interface: DI, rst_pin: &mut PinDriver<'_, P, Output>) -> Result<Self, OledErorr> {
        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

        // Reset screen
        rst_pin.set_low()
            .map_err(|_| OledErorr::ResetLinefailed)?;
        FreeRtos::delay_ms(15);
        rst_pin.set_high()
            .map_err(|_| OledErorr::ResetLinefailed)?;
        FreeRtos::delay_ms(15);

        // Initlialization screen controler
        display.init()
            .map_err(|_| OledErorr::Initializationfailed)?;

        let text_style = MonoTextStyleBuilder::new()
            .font(&ascii::FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        Ok(Self { display, text_style })
    }

    /// Clear internal screen buffer, set all pixels to black
    ///
    /// **Note:** This function only modifies ESP RAM. To see
    /// result on screen, cal the [`Self::refresh()`] function.
    pub fn clear(&mut self) -> Result<(), OledErorr> {
        self.display.clear(BinaryColor::Off)
            .map_err(|_| OledErorr::ClearFailed)?;

        Ok(())
    }

    /// Draw text in internal graphics buffer at specified coordinates.
    ///
    /// # Arguments
    /// `text` - Any str to show
    /// `x` - Initial position on horizontal axis (0 - 127)
    /// `y` - Initial position on vertical axis (0 - 63)
    ///
    /// **Note:** This function only modifies ESP RAM. To see
    /// result on screen, cal the [`Self::refresh()`] function.
    pub fn show_text(&mut self, text: &str, x: i32, y: i32) -> Result<(), OledErorr> {
        Text::new(text, Point::new(x, y), self.text_style)
            .draw(&mut self.display)
            .map_err(|_| OledErorr::DrawFailed)?;

        Ok(())
    }

    /// Transfers contents of RAM buffer directly to SSD1306 controller.
    /// Updates image visible to user
    pub fn refresh(&mut self) -> Result<(), OledErorr> {
        self.display.flush()
            .map_err(|_| OledErorr::FlushFailed)?;

        Ok(())
    }
}

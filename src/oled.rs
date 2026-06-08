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

pub struct OledDisplay<DI> {
    display: Ssd1306<DI, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>,
    text_style: MonoTextStyle<'static, BinaryColor>,
}

impl<DI> OledDisplay<DI>
where
    DI: WriteOnlyDataCommand,
{
    pub fn new<P: Pin>(interface: DI, rst_pin: &mut PinDriver<'_, P, Output>) -> Result<Self> {
        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

        rst_pin.set_low().map_err(|e| anyhow::anyhow!("Set low error: {:?}", e))?;
        FreeRtos::delay_ms(15);
        rst_pin.set_high().map_err(|e| anyhow::anyhow!("Set high error: {:?}", e))?;
        FreeRtos::delay_ms(15);

        display.init().map_err(|e| anyhow::anyhow!("Display init error: {:?}", e))?;

        let text_style = MonoTextStyleBuilder::new()
            .font(&ascii::FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        Ok(Self { display, text_style })
    }

    pub fn clear(&mut self) -> Result<()> {
        self.display.clear(BinaryColor::Off)
            .map_err(|e| anyhow::anyhow!("Clear error: {:?}", e))?;

        Ok(())
    }

    pub fn show_text(&mut self, text: &str, x: i32, y: i32) -> Result<()> {
        Text::new(text, Point::new(x, y), self.text_style)
            .draw(&mut self.display)
            .map_err(|e| anyhow::anyhow!("Draw error: {:?}", e))?;

        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.display.flush()
            .map_err(|e| anyhow::anyhow!("Flush error: {:?}", e))?;

        Ok(())
    }
}

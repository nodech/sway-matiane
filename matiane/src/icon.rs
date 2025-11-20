// Generated automatically by iced_fontello at build time.
// Do not edit manually. Source: ../fonts/icons.toml
// 144f0c14b306b9d3e23c75b6ec2e7e8a4a18b9ea772e8c0dd97e185c87860ada
use iced::Font;
use iced::widget::{Text, text};

pub const FONT: &[u8] = include_bytes!("../fonts/icons.ttf");

pub fn clock<'a>() -> Text<'a> {
    icon("\u{1F554}")
}

pub fn globe<'a>() -> Text<'a> {
    icon("\u{1F30E}")
}

pub fn moon<'a>() -> Text<'a> {
    icon("\u{F186}")
}

pub fn sun<'a>() -> Text<'a> {
    icon("\u{F185}")
}

fn icon(codepoint: &str) -> Text<'_> {
    text(codepoint).font(Font::with_name("icons"))
}

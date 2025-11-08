use crossterm::style;

#[allow(dead_code)]
pub fn parse_ansi(ansi: &str) -> Option<style::Color> {
    style::Color::parse_ansi(&format!("5;{ansi}"))
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Color(#[allow(unused)] pub style::Color);

impl FromStr for Color {
    type Err = &'static str;

    fn from_str(ansi: &str) -> Result<Self, Self::Err> {
        if let Some(c) = parse_ansi(ansi) {
            Ok(Color(c))
        } else {
            Err("Invalid color")
        }
    }
}

use crate::prelude::*;

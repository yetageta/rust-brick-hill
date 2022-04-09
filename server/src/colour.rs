// https://docs.rs/hex-rgb/latest/src/hex_rgb/lib.rs.html
use std::num::ParseIntError;

#[derive(Debug)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn new(hex_code: &str) -> Result<Color, String> {
        if hex_code.is_empty() {
            return Err("empty".to_string());
        }

        // remove # from hex_code
        let hex_code = if hex_code.starts_with('#') {
            crop_letters(hex_code, 1)
        } else {
            hex_code
        };

        // convert shorthand RGB hexcode to RRGGBB
        let hex_code = if hex_code.len() == 3 {
            repeat_letters(hex_code, 1)
        } else {
            hex_code.to_owned()
        };

        if hex_code.len() % 2 != 0 {
            return Err("invalid".to_string());
        }

        let decoded_values = decode_hex(&hex_code).unwrap_or_default();
        if decoded_values.is_empty() || decoded_values.len() > 4 {
            return Err("invalid".to_string());
        }

        let color = Color {
            red: decoded_values[0],
            green: decoded_values[1],
            blue: decoded_values[2],
        };

        Ok(color)
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Color) -> bool {
        self.red == other.red && self.green == other.green && self.blue == other.blue
    }
}

fn crop_letters(s: &str, pos: usize) -> &str {
    match s.char_indices().nth(pos) {
        Some((pos, _)) => &s[pos..],
        None => "",
    }
}

fn repeat_letters(s: &str, repetitions: i32) -> String {
    let mut output = String::from("");
    for char in s.chars() {
        for _ in 0..=repetitions {
            output.push(char);
        }
    }

    output
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn convert_hexcode_to_rgb(hex_code: String) -> Result<Color, String> {
    match Color::new(&hex_code) {
        Ok(color) => Ok(color),
        Err(e) => Err(e),
    }
}

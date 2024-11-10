// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Parsing example
//!
//! Outputs debug strings for the parse to stdout
//!
//! Typical usage:
//!
//! ```sh
//! cargo run --example parse 'oklab(0.5 0.2 0)'
//! ```

use color::{AlphaColor, Hwb, Lab, Srgb};

fn main() {
    let arg = std::env::args().nth(1).expect("give color as arg");
    match color::parse_color(&arg) {
        Ok(color) => {
            println!("display: {color}");
            println!("debug: {color:?}");
            let srgba: AlphaColor<Srgb> = color.to_alpha_color();
            println!("{srgba:?}");
            let lab: AlphaColor<Lab> = color.to_alpha_color();
            println!("{lab:?}");
            let hwb: AlphaColor<Hwb> = color.to_alpha_color();
            println!("{hwb:?}");
        }
        Err(e) => println!("error: {e}"),
    }
}

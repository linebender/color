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

use color::{AlphaColor, CssColor, Srgb};

fn main() {
    let arg = std::env::args().nth(1).expect("give color as arg");
    match color::parse_color(&arg) {
        Ok(color) => {
            println!("display: {color}");
            println!("debug: {color:?}");
            let tagged = CssColor::to_tagged_color(color);
            let srgba: AlphaColor<Srgb> = tagged.to_alpha_color();
            println!("{srgba:?}");
        }
        Err(e) => println!("error: {e}"),
    }
}

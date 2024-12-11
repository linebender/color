// Copyright 2024 the Color Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Gradient example
//!
//! Outputs a test page to stdout.
//!
//! Typical usage:
//!
//! ```sh
//! cargo run --example gradient 'oklab(0.5 0.2 0)' 'rgb(0, 200, 0, 0.8)' oklab
//! ```

use color::{gradient, ColorSpaceTag, DynamicColor, GradientIter, HueDirection, Srgb};

fn main() {
    let mut args = std::env::args().skip(1);
    let c1_s = args.next().expect("give color as arg");
    let c1 = color::parse_color(&c1_s).expect("error parsing color 1");
    let c2_s = args.next().expect("give 2 colors as arg");
    let c2 = color::parse_color(&c2_s).expect("error parsing color 2");
    let cs_s_raw = args.next();
    let cs_s = cs_s_raw.as_deref().unwrap_or("srgb");
    let cs: ColorSpaceTag = cs_s.parse().expect("error parsing color space");
    let gradient: GradientIter<Srgb> = gradient(c1, c2, cs, HueDirection::default(), 0.02);
    println!("<!DOCTYPE html>");
    println!("<html>");
    println!("<head>");
    println!("<style>");
    println!("div.g {{ height: 100px }}");
    println!("#basic {{ background: linear-gradient(to right in {cs_s}, {c1_s}, {c2_s}) }}");
    print!("#ours {{ background: linear-gradient(to right");
    for (t, stop) in gradient {
        print!(
            ", {} {}%",
            DynamicColor::from_alpha_color(stop.un_premultiply()),
            t * 100.0
        );
    }
    println!(") }}");
    println!("</style>");
    println!("</head>");
    println!("<body>");
    println!("<div>{c1_s} {c2_s} {cs_s}</div>");
    println!("<div class='g' id='basic'></div>");
    println!("<div class='g' id='ours'></div>");
    println!("</body>");
    println!("</html>");
}

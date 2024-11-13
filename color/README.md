<div align="center">

# Color

**TODO: Add tagline**

[![Linebender Zulip, #color channel](https://img.shields.io/badge/Linebender-%23color-blue?logo=Zulip)](https://xi.zulipchat.com/#narrow/channel/466849-color)
[![dependency status](https://deps.rs/repo/github/linebender/color/status.svg)](https://deps.rs/repo/github/linebender/color)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](#license)
[![Build status](https://github.com/linebender/color/workflows/CI/badge.svg)](https://github.com/linebender/color/actions)
[![Crates.io](https://img.shields.io/crates/v/color.svg)](https://crates.io/crates/color)
[![Docs](https://docs.rs/color/badge.svg)](https://docs.rs/color)

</div>

<!-- We use cargo-rdme to update the README with the contents of lib.rs.
To edit the following section, update it in lib.rs, then run:
cargo rdme --workspace-project=color --heading-base-level=0
Full documentation at https://github.com/orium/cargo-rdme -->

<!-- Intra-doc links used in lib.rs should be evaluated here. 
See https://linebender.org/blog/doc-include/ for related discussion. -->
[libm]: https://crates.io/crates/libm

<!-- cargo-rdme start -->

Color is a Rust crate which implements color space conversions, targeting at least CSS4 color.

## Features

- `std` (enabled by default): Get floating point functions from the standard library (likely using your target's libc).
- `libm`: Use floating point implementations from [libm][].

At least one of `std` and `libm` is required; `std` overrides `libm`.

<!-- cargo-rdme end -->

## Minimum supported Rust Version (MSRV)

This version of Color has been verified to compile with **Rust 1.82** and later.

Future versions of Color might increase the Rust version requirement.
It will not be treated as a breaking change and as such can even happen with small patch releases.

<details>
<summary>Click here if compiling fails.</summary>

As time has passed, some of Color's dependencies could have released versions with a higher Rust requirement.
If you encounter a compilation issue due to a dependency and don't want to upgrade your Rust toolchain, then you could downgrade the dependency.

```sh
# Use the problematic dependency's name and version
cargo update -p package_name --precise 0.1.1
```

</details>

## Community

[![Linebender Zulip](https://img.shields.io/badge/Xi%20Zulip-%23color-blue?logo=Zulip)](https://xi.zulipchat.com/#narrow/channel/466849-color)

Discussion of Color development happens in the [Linebender Zulip](https://xi.zulipchat.com/), specifically the [#color channel](https://xi.zulipchat.com/#narrow/channel/466849-color).
All public content can be read without logging in.

## License

Licensed under either of

- Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Contributions are welcome by pull request. The [Rust code of conduct] applies.
Please feel free to add your name to the [AUTHORS] file in any substantive pull request.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.

[Rust Code of Conduct]: https://www.rust-lang.org/policies/code-of-conduct
[AUTHORS]: ./AUTHORS

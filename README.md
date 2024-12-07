# esptools

[Esp tools](https://github.com/espressif/esptool) (`esptool`, `espsecure` and `espefuse`) bundler.

[![CI](https://github.com/ivmarkov/esptools/actions/workflows/ci.yml/badge.svg)](https://github.com/ivmarkov/esptools/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/esptools.svg)](https://crates.io/crates/esptools)
[![Matrix](https://img.shields.io/matrix/esp-rs:matrix.org?label=join%20matrix&color=BEC5C9&logo=matrix)](https://matrix.to/#/#esp-rs:matrix.org)

Bundles the ESP tools as a Rust library.

**Q:** Why do I need it? Espressif already distributes [self-contained pre-built executables for all the major platforms](https://github.com/espressif/esptool/releases/tag/v4.8.1)?

**A:** To use these from a `build.rs` script or other Rust code, you still have to download the .ZIP corresponding to your OS, extract it
   and then put the executables in your `$PATH` before being able to call the tool of your choice.
   This is exactly what this crate automates!

---
**NOTE**

`esptools` will only run on those platforms where Espressif supplies pre-built binaries. I.e. Linux / MacOS / Windows X86_64 as well as Linux ARM64 and ARM32.

---

## Examples

### Command line

```sh
cargo install --force --git https://github.com/ivmarkov/esptools
esptools efuse -h
```

### Library

```rust
use esptools::{Tool, Tools};

fn main() -> anyhow::Result<()> {
    let tools = Tools::mount()?;

    tools.exec(Tool::EspEfuse, &["-h"])?;

    Ok(())
}
```

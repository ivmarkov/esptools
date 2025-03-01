# esptools

[Esp tools](https://github.com/espressif/esptool) (`esptool`, `espsecure`, `espefuse` and `espidfnvs`) bundler.

[![CI](https://github.com/ivmarkov/esptools/actions/workflows/ci.yml/badge.svg)](https://github.com/ivmarkov/esptools/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/esptools.svg)](https://crates.io/crates/esptools)
[![Matrix](https://img.shields.io/matrix/esp-rs:matrix.org?label=join%20matrix&color=BEC5C9&logo=matrix)](https://matrix.to/#/#esp-rs:matrix.org)

Bundles the ESP tools as a Rust library.

**Q:** Why do I need it? Espressif already provides [self-contained pre-built executables for all the major platforms](https://github.com/espressif/esptool/releases/tag/v4.8.1)?

**A:** To use these from a `build.rs` script or other Rust code, you still have to download the archive corresponding to your OS, extract it
   and then put the executables in your `$PATH` before being able to call the tool of your choice.
   This is exactly what this crate automates!

---
**NOTE: Supported platforms**

`esptools` will only run on those platforms where Espressif supplies pre-built binaries. I.e. Linux / MacOS / Windows X86_64 as well as Linux ARM64 and ARM32.

**NOTE: Licensing**

While the `esptools` crate is licensed under Apache + MIT (as usual with Rust), the bundled (and thus distributed) binaries of `esptool`, `espsecure` and `espefuse` [are licensed under the **GPL v2**](https://github.com/espressif/esptool/blob/master/LICENSE). (This does not apply to `espidfnvs` 
a.k.a. `esp-idf-nvs-partition-gen`, which is a separate repo [licensed under the Apache license](https://github.com/espressif/esp-idf-nvs-partition-gen/blob/main/LICENSE).)

With that said, [distributing those should be OK](https://www.reddit.com/r/opensource/comments/nok8lg/include_binaries_of_a_gpl_licensed_program/), as we are providing a [link](https://github.com/espressif/esptool) to the upstream Espressif GIT repo containing the binaries' source code, as well as [the download location of the binaries themselves](https://github.com/espressif/esptool/releases/tag/v4.8.1).

Let us know if you think otherwise!

If you distribute - outside of your premises and e.g. the factory flashing your chips - a binary using this library, you might want to include these links in your binary documentation though!

---

## Examples

### Command line

```sh
cargo install esptools
esptools efuse -h
```

### Library

```rust
fn main() -> anyhow::Result<()> {
    esptools::Tool::EspEfuse.mount()?.exec(&["-h"])
}
```

## Cross-building for other targets than the host one

`esptools` is pure-Rust so you only need a [linker for your cross-target](https://capnfabs.net/posts/cross-compiling-rust-apps-raspberry-pi/) and a C cross toolchain for the few dependencies that still need to compile custom C files (`ring`).

Sample ways to cross-compile:

(If `cargo` greets you with a "note: the `XXX` target may not be installed" error, install the target first with `rustup target add XXX`.)

### With [`cargo-zigbuild`](https://github.com/rust-cross/cargo-zigbuild) 

```sh
cargo install cargo-zigbuild
pip3 install zig
cargo zigbuild --target aarch64-unknown-linux-gnu # rPI 4+
```

> Note: does not support cross-compiling to Windows. For Windows, use some of the other options.

### With [`cargo-xwin`](https://github.com/rust-cross/cargo-xwin) 

```sh
cargo install cargo-xwin
cargo xwin build --target x86_64-pc-windows-msvc
```

### With [`cross`](https://hackernoon.com/building-a-wireless-thermostat-in-rust-for-raspberry-pi-part-2) 

```sh
cargo install cross
cross build --target=x86_64-pc-windows-gnu # For e.g. Windows; Windows MSVC is not supported, only the GNU target
```

> Note: needs Docker or Podman pre-installed.

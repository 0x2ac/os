# OS

Following [Phil Opp's Guide](https://os.phil-opp.com/).

## Setup

Requires a few things to be installed:

```shell
rustup toolchain install nightly
rustup component add llvm-tools-preview
cargo install bootimage
```

If you want to use `cargo run` to directly boot into an emulator you also need `qemu` installed on your system. Find a way to get it for your platform on their [download page](qemu.org/download/).

```shell
# To build
cargo bootimage

# To build and open in qemu
cargo run
```

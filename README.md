# picontrol-rs

Rust version of the PiControl driver for the Revolution Pi.

## An example: pitestrs

A command line tool to control the Pi Control process image is in the file [pitestrs.rs](src/bin/pitestrs.rs).
The executable can be cross-compiled by launching `./build_pi.sh`.
See below how to enable cross compilation.

## How to generate the Rust FFI bindings to C

1. Use bindgen binary directly:

   ```bash
   TARGET=armv7-unknown-linux-gnueabihf bindgen -o src/picontrol.rs kunbus/interface/piControl/wrapper.h  -- -I`pwd`/kunbus/interface/piControl
   ```

2. Use bindgen with a build.rs file

## Rust cross-compilation

### Cross project with Docker

See: https://github.com/rust-embedded/cross


### Same operating system

See: https://medium.com/@wizofe/cross-compiling-rust-for-arm-e-g-raspberry-pi-using-any-os-11711ebfc52b

```bash
source $HOME/.cargo/env

sudo apt-get install -qq gcc-arm-linux-gnueabihf
rustup target add armv7-unknown-linux-gnueabihf
```

Configure Cargo for cross-compilation

```bash
mkdir -p ~/.cargo
# > should not be included when pasting
cat >> ~/.cargo/config << EOF
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
EOF

# you may need to install this on Ubuntu 18.06
sudo apt-get install g++-multilib libc6-dev-i386
sudo apt-get install -qq gcc-arm-linux-gnueabihf
```

You can now compile:

```bash
cargo build --target=armv7-unknown-linux-gnueabihf
```
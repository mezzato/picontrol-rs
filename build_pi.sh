#!/bin/bash

# cargo build --release --target=armv7-unknown-linux-gnueabihf
# cargo build --release --target=arm-unknown-linux-gnueabi
# cargo build --release --target=arm-unknown-linux-gnueabi

cross build --release --target=armv7-unknown-linux-gnueabihf

# RUSTFLAGS=-g cross build --release --target=armv7-unknown-linux-gnueabihf
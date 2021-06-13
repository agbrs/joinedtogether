#!/bin/bash

cargo build --release
arm-none-eabi-objcopy -O binary target/thumbv4t-none-eabi/release/joinedtogether joinedtogether.gba
gbafix -p joinedtogether.gba

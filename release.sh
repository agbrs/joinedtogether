#!/bin/bash

cargo build --release
arm-none-eabi-objcopy -O binary target/thumbv4t-none-eabi/release/joinedtogether thehatchoosesthewizard.gba
gbafix -p thehatchoosesthewizard.gba

#!/bin/bash

cargo build --release

mkdir -p target/final-zip
rm -rf traget/final-zip/the-hat-chooses-the-wizard
cp -rv release-data/the-hat-chooses-the-wizard target/final-zip

arm-none-eabi-objcopy -O binary target/thumbv4t-none-eabi/release/joinedtogether target/final-zip/the-hat-chooses-the-wizard/thehatchoosesthewizard.gba
gbafix -p target/final-zip/the-hat-chooses-the-wizard/thehatchoosesthewizard.gba

rm -f target/thehatchoosesthewizard.zip
(cd target/final-zip && zip -r ../thehatchoosesthewizard.zip the-hat-chooses-the-wizard)

#!/bin/bash -e

TARGET=arm-unknown-linux-gnueabihf
USER=pi

#todo validate cross
cargo install -f cross

cross build --release --target $TARGET

scp -r ./target/$TARGET/release/carto $USER@192.168.0.191:/home/pi
ssh $USER@192.168.0.191 /home/pi/carto


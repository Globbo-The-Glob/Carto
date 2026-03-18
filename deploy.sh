#!/bin/bash -e

TARGET=armv7-unknown-linux-gnueabihf
USER=pi

#todo validate cross
cargo install -f cross

cross build --release --target $TARGET

scp -r ./target/$TARGET/release/carto $USER@192.168.0.211:/home/pi
ssh $USER@192.168.0.211 /home/pi/carto


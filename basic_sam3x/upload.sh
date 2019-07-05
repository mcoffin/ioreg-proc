#!/bin/bash
set -e

bin_file=${1:-../target/thumbv7m-none-eabi/release/basic_sam3x.bin}

stty -F "/dev/ttyACM0" raw ispeed 1200 ospeed 1200 cs8 -cstopb ignpar eol 255 eof 255
printf "\x00" > "/dev/ttyACM0"

sleep 1

bossac -i -d -e -w -v -b -R "$bin_file"

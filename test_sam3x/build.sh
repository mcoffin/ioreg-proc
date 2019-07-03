#!/bin/bash
set -e
set -x

device_path=${1:-/dev/ttyACM0}

upload() {
	device=${1:-/dev/ttyACM0}
	fw_file=${2:-../target/firmware.bin}
	stty -F "$device" raw ispeed 1200 ospeed 1200 cs8 -cstopb ignpar eol 255 eof 255
	printf "\x00" > "$device"
	sleep 1
	# /home/mcoffin/.arduino15/packages/arduino/tools/bossac/1.6.1-arduino/bossac -i -d --port=ttyACM0 -U false -e -w -v -b "$fw_file" -R
	bossac -i -d -e -w -v -b -R "$fw_file"
}

RUSTFLAGS="-C target-cpu=cortex-m3 -C link-args=-Tlayout.ld" \
	cargo xbuild \
	--target thumbv7m-none-eabi \
	--release

if [ -e ../target/firmware.bin ]; then
	rm ../target/firmware.bin
fi

arm-none-eabi-objcopy -O binary ../target/thumbv7m-none-eabi/release/test_sam3x ../target/firmware.bin

upload "$device_path" ../target/firmware.bin

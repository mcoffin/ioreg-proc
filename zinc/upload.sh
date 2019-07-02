#!/bin/bash
set -x
set -e

example_name=${1:-blink_sam3x}
port=${2:-/dev/ttyACM0}
target=thumbv7m-none-eabi-zinc

dump_binary() {
	if [ -e ../target/firmware.bin ]; then
		rm ../target/firmware.bin
	fi
	arm-none-eabi-objcopy -O binary "../target/$target/release/examples/$1" ../target/firmware.bin
}

programming_upload() {
	fw_file=${2-../target/firmware.bin}
	stty -F "$1" raw ispeed 1200 ospeed 1200 cs8 -cstopb ignpar eol 255 eof 255
	bash -c "printf \"\x00\" > \"$1\""
	sleep 1
	bossac --write --verify --boot -R $fw_file
}

RUSTFLAGS="-C link-args=-Tlayout.ld" \
	RUST_TARGET_PATH="$(pwd)/.." \
	cargo xbuild \
	--target=../thumbv7m-none-eabi-zinc.json \
	--release \
	--example "$example_name" \
	--features mcu_sam3x
dump_binary $example_name
programming_upload "$port"

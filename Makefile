.PHONY: build run build-release run-release

build:
	cargo build

run:
	cargo run

build-release:
	cargo build --release

run-release:
	cargo run --release

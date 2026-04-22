#!/usr/bin/env sh
set -eu

kernel="$1"

exec qemu-system-aarch64 \
  -machine virt \
  -cpu cortex-a72 \
  -m 256M \
  -smp 1 \
  -nographic \
  -monitor none \
  -serial stdio \
  -netdev user,id=net0,hostfwd=tcp:127.0.0.1:8080-:8080 \
  -device virtio-net-device,netdev=net0,bus=virtio-mmio-bus.0,mac=52:54:00:12:34:56 \
  -device loader,file="$kernel",cpu-num=0

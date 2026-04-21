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
  -device loader,file="$kernel",cpu-num=0

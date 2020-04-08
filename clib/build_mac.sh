#!/bin/sh
cargo build
gcc -o test -g test.c -I include target/debug/libdp3t.a -lpthread -ldl -framework Security

@echo off

set RUSTFLAGS=-Awarnings
cargo build --lib --release


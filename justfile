#!/usr/bin/env just --justfile

set windows-shell := ["powershell.exe"]

# Clean build
clean-build:
    cargo clean
    cargo build --release

# Clean install (includes removing existing installation)
clean-install:
    remove-Item "C:\Users\ronak\.rshare"
    cargo install --path .

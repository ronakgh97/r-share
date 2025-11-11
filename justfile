#!/usr/bin/env just --justfile

# Clean build (for release)
clean-build:
    cargo clean
    cargo build --release

# Clean install (includes removing existing installation)
clean-install:
    remove-Item "~\.rshare"
    cargo install --path .

clean-docker:
    docker ps -a -q --filter "name=rshare" | xargs -r docker rm -f
    docker images -a --filter=reference='*rshare*' -q | xargs -r docker rmi -f
    docker-compose up -d --build
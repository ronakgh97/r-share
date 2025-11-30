# R-Share

[![Rust](https://img.shields.io/badge/Rust-1.82+-orange.svg)](https://www.rust-lang.org/)
[![Spring Boot](https://img.shields.io/badge/Spring%20Boot-3.5.7-brightgreen.svg)](https://spring.io/projects/spring-boot)
[![Netty](https://img.shields.io/badge/Netty-4.1-blue.svg)](https://netty.io/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**R-Share** is a simple secure, **blazingly-fast** and lightweight relay file sharing tool with **Ed25519 cryptographic
signatures** and **SHA256 integrity verification**. Built with Rust CLI clients and a Spring Boot + Netty relay server.

## Current State

- Core CLI commands implemented: `init`, `serve`, `listen`, `trust`, `relay`, `health`
- Ed25519 key generation and signature verification working
- End-to-end encryption with X25519 key exchange and AES-256-GCM
- HTTP + Socket relay protocol operational with session-based pairing
- Contact management via JSON-based trust system
- Memory-mapped file hashing for fast SHA256 integrity checks

### DEMO
![Demo](demo.gif)

## Roadmap

- Transfer history command and persistent logs
- Multi-file and directory transfer support
- HTTPS support for relay API communication
- `me` command to view own identity and fingerprint

## Contributions

Contributions are welcome! Feel free to open issues or submit pull requests.

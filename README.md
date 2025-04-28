# Zed SOPS Encrypt/Decrypt Extension

Automatically decrypts SOPS-encrypted files on open and re-encrypts on save/close within the Zed IDE, with in-editor notifications.

## Features

- Detects SOPS-managed files (`.enc.yaml`, `.enc.json`, `.sops`) and auto-decrypts on open.
- Auto re-encrypts on save/close.
- Shows rich notifications in Zed’s UI for success and error events.

## Prerequisites

- Rust (via `rustup`)
- `sops` on your `PATH`
- Zed ≥ v0.131.0

## Installation

1. Clone the repo:
```bash
git clone https://github.com/your-org/zed-sops-extension.git
cd zed-sops-extension
cargo build --release
```

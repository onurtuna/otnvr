#!/usr/bin/env bash
set -euo pipefail

function info() {
    printf '[INFO] %s\n' "$*" >&2
}

function warn() {
    printf '[WARN] %s\n' "$*" >&2
}

function require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        warn "Missing required command: $1"
        return 1
    fi
    return 0
}

info "Installing runtime and build dependencies for OtNvr"

UNAME="$(uname -s)"
case "${UNAME}" in
    Darwin)
        if ! xcode-select -p >/dev/null 2>&1; then
            warn "Command Line Tools for Xcode are not installed. Run 'xcode-select --install' first."
            exit 1
        fi

        if ! /usr/bin/xcodebuild -license check >/dev/null 2>&1; then
            warn "You must accept the Xcode license: run 'sudo xcodebuild -license' once before continuing."
            exit 1
        fi

        if command -v brew >/dev/null 2>&1; then
            info "Using Homebrew to install FFmpeg and supporting tools"
            brew update
            brew install ffmpeg pkg-config || true
        else
            warn "Homebrew is not installed. Install it from https://brew.sh/ or install FFmpeg manually."
            exit 1
        fi
        ;;
    Linux)
        if command -v apt-get >/dev/null 2>&1; then
            info "Using apt to install FFmpeg and build-essential packages"
            sudo apt-get update
            sudo apt-get install -y ffmpeg pkg-config build-essential clang
        elif command -v dnf >/dev/null 2>&1; then
            info "Using dnf to install FFmpeg and development tools"
            sudo dnf install -y ffmpeg ffmpeg-devel pkg-config clang make gcc
        else
            warn "Unsupported Linux distribution. Install FFmpeg, pkg-config, clang, and build tools manually."
            exit 1
        fi
        ;;
    *)
        warn "Unsupported platform: ${UNAME}. Install FFmpeg, pkg-config, and a C toolchain manually."
        exit 1
        ;;
esac

if ! require_command cargo; then
    warn "Rust toolchain is missing. Install rustup from https://rustup.rs/."
    exit 1
fi

info "Fetching Rust dependencies (this may take a moment)"
cargo fetch

info "All dependencies installed. Create a JSON config (see config.example.json) and run 'cargo run <config.json>'."

# Brotli-rs - Brotli decompression in pure, safe Rust

[![Build Status](https://api.travis-ci.org/ende76/brotli-rs.png?branch=master)](https://travis-ci.org/ende76/brotli-rs) ![Works on stable](https://img.shields.io/badge/works%20on-stable-green.svg) ![Works on beta](https://img.shields.io/badge/works%20on-beta-yellow.svg) ![Works on nightly](https://img.shields.io/badge/works%20on-nightly-lightgrey.svg)

[Documentation](http://ende76.github.io/brotli-rs/brotli/)

Compression provides a <Read>-struct to wrap a Brotli-compressed stream. A consumer is thus able to read a compressed stream with usual convenience.

# Changelog

v0.2.0 -> v0.3.0
----------------

- renamed crate compression -> brotli
- restructured modules to avoid redundant paths like brotli::brotli::Decompressor (now it's just brotli::Decompressor)


v0.1.0 -> v0.2.0
----------------

- Decompressor::new() now accepts a Read, as opposed to a BitReader.
# Brotli-rs - Brotli decompression in pure, safe Rust

[![Build Status](https://api.travis-ci.org/ende76/brotli-rs.png?branch=master)](https://travis-ci.org/ende76/brotli-rs) ![Works on stable](https://img.shields.io/badge/works%20on-stable-green.svg) ![Works on beta](https://img.shields.io/badge/works%20on-beta-yellow.svg) ![Works on nightly](https://img.shields.io/badge/works%20on-nightly-lightgrey.svg)

[Documentation](http://ende76.github.io/brotli-rs/brotli/)

Compression provides a <Read>-struct to wrap a Brotli-compressed stream. A consumer is thus able to read a compressed stream with usual convenience.

## Changelog

###v0.3.1 -> v0.3.4
----------------

Fair amount of internal small improvements, improving code quality. Fixed a couple of cases where invalid streams would lead to panics and/or infinite loops (Thanks, [Corey](https://github.com/frewsxcv)!).


###v0.3.0 -> v0.3.1
----------------

This is only a minor version bump, with no breakage in usage, but it's exciting nonetheless!

In Brotli, a lot of work is done with and by prefix codes. Through a change in the internal representation of prefix codes, it was possible to speed the reference benchmark time by a factor of ~7. The benchmark decompresses the contents of the file data/monkey.compressed.

- With linked-list-based, recursive tree implementation:  
test bench_monkey              ... bench:     __866,888 ns__/iter (+/- 58,119)

- With array-based, iterative tree implementation, before max-depth constraint:  
test bench_monkey              ... bench:     __704,282 ns__/iter (+/- 220,068)

- With array-based, iterative tree implementation, with max-depth constraint:  
test bench_monkey              ... bench:     __120,745 ns__/iter (+/- 16,627)


###v0.2.0 -> v0.3.0
----------------

- renamed crate compression -> brotli
- restructured modules to avoid redundant paths like brotli::brotli::Decompressor (now it's just brotli::Decompressor)


###v0.1.0 -> v0.2.0
----------------

- Decompressor::new() now accepts a Read, as opposed to a BitReader.
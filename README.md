# Brotli-rs - Brotli decompression in pure, safe Rust

[![Build Status](https://api.travis-ci.org/ende76/brotli-rs.png?branch=master)](https://travis-ci.org/ende76/brotli-rs) ![Works on stable](https://img.shields.io/badge/works%20on-stable-green.svg) ![Works on beta](https://img.shields.io/badge/works%20on-beta-yellow.svg) ![Works on nightly](https://img.shields.io/badge/works%20on-nightly-lightgrey.svg)

[Documentation](http://ende76.github.io/brotli-rs/brotli/)

Compression provides a \<Read\>-struct to wrap a Brotli-compressed stream. A consumer is thus able to read a compressed stream with usual convenience.

## Changelog


###v0.3.21 -> v0.3.22
----------------

Bug with metablock structure not getting fully reset when encountering a new metablock in the brotli stream, reported and fixed by [Daniel](https://github.com/danielrh).

###v0.3.20 -> v0.3.21
----------------

Bug with multiple metablocks, reported and fixed by [Daniel](https://github.com/danielrh).

###v0.3.19 -> v0.3.20
----------------

Worked around feature gate issue in nightly. (Thanks, [Corey](https://github.com/frewsxcv)!)

###v0.3.18 -> v0.3.19
----------------

Removed 64k big Pseudo-Code lookup table, because creating this trivial table probably took more time than making the calculcation on the fly. (Bench tests seem to suggest a 1% time savings without the lookup table)

###v0.3.17 -> v0.3.18
----------------

Fixed case where a simple prefix code could have duplicate symbols.

###v0.3.16 -> v0.3.17
----------------

Fixed case where a complex prefix code could have an incorrect checksum on its runlength code.

###v0.3.15 -> v0.3.16
----------------

- Fixed incorrect calculation of alphabet size for distance code.
- Fixed evaluation where streams with excessive insert/copy lengths could be rejected early. 

###v0.3.14 -> v0.3.15
----------------

Fixed injection of invalid symbols in simple prefix code.

###v0.3.13 -> v0.3.14
----------------

Fixed invalid block-type in switch command. (Thanks, [Corey](https://github.com/frewsxcv)!).

###v0.3.12 -> v0.3.13
----------------

Fixed uncaught non-positive distances. (Thanks, [Corey](https://github.com/frewsxcv)!).

###v0.3.11 -> v0.3.12
----------------

Fixed uncaught zero-byte in word transformation. (Thanks, [Corey](https://github.com/frewsxcv)!).

###v0.3.10 -> v0.3.11
----------------

Fixed possible arithmetic overflow in word transformation. (Thanks, [Corey](https://github.com/frewsxcv)!).

###v0.3.9 -> v0.3.10
----------------

Fixed incorrect type for runlength code. (Thanks, [Corey](https://github.com/frewsxcv)!).

###v0.3.8 -> v0.3.9
----------------

Fixed incorrect array index bound check in tree lookup. (Thanks, [Corey](https://github.com/frewsxcv)!).

###v0.3.7 -> v0.3.8
----------------

Fixed some value range checks on block types and ntree*. (Thanks, [Corey](https://github.com/frewsxcv)!).

###v0.3.6 -> v0.3.7
----------------

Went over "unreachable!()" statements, analyzed, and handled error condition properly, if they were reachable through invalid data.

###v0.3.5 -> v0.3.6
----------------

Fixed a case where an invalid prefix code with all-zero codelengths could create an index-out-of-bounds panic. (Thanks, [Corey](https://github.com/frewsxcv)!).

###v0.3.4 -> v0.3.5
----------------

Fixed a case where an invalid insert-and-copy-length-code would produce a panic. (Thanks, [Corey](https://github.com/frewsxcv)!).

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
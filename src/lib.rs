#![deny(missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts, trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces, unused_qualifications)]
//! brotli-rs provides Read adapter implementations the Brotli compression scheme.
//!
//! This allows a consumer to wrap a Brotli-compressed Stream into a Decompressor,
//! using the familiar methods provided by the Read trait for processing
//! the uncompressed stream.

/// bitreader wraps a Read to provide bit-oriented read access to a stream.
mod bitreader;
/// brotli provides a Read implementation to wrap a brotli stream, for convenient decompresssion.
pub mod brotli;
mod huffman;
/// ringbuffer provides a data structure RingBuffer that uses a single, fixed-size buffer as if it were connected end-to-end.
/// This structure lends itself easily to buffering data streams.
mod ringbuffer;
#![deny(missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts, trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces, unused_qualifications)]
//! compression provides Read adapter implementations for common compression schemes, most notably
//! for Brotli. This allows a consumer to wrap a Brotli-compressed Stream into a Decompressor,
//! allowing the consumer to use the familiar methods provided by the Read trait for consuming
//! the uncompressed stream.



/// bitreader wraps a Read to provide bit-oriented read access to a stream.
pub mod bitreader;
/// brotli provides a Read implementation to wrap a brotli stream, for convenient decompresssion.
pub mod brotli;
/// deflate provides a Read implementation to wrap a deflate stream, for convenient decompresssion.
pub mod deflate;
/// gzip provides a Read implementation to wrap a gzip stream, for convenient decompresssion.
pub mod gzip;
mod huffman;
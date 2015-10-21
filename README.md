# Compression - A decompressor implementation for the Brotli specification

compression provides an implementation of a Brotli decompressor. The struct compression::brotli::Decompressor implements std::io::Read. That way, a consumer can wrap a Brotli-compressed stream in a Decompressor, reading the uncompressed stream conveniently through the Read interface.
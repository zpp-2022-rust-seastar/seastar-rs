#pragma once

#include "cxx_async_futures.hh"
#include <seastar/core/file.hh>
#include <seastar/core/file-types.hh>
#include <seastar/core/seastar.hh>

struct OpenOptions;

namespace seastar_ffi {
namespace file {

using file_t = seastar::file;
using open_flags = seastar::open_flags;

// Creates `seastar::open_flags` from Rust's OpenOptions.
open_flags parse_options(const OpenOptions& opts);

VoidFuture open_dma(std::unique_ptr<file_t>& file, rust::str name, const OpenOptions& opts);

IntFuture read_dma(const std::unique_ptr<file_t>& file, uint8_t* buffer, uint64_t size, uint64_t pos);

IntFuture write_dma(const std::unique_ptr<file_t>& file, uint8_t* buffer, uint64_t size, uint64_t pos);

VoidFuture flush(const std::unique_ptr<file_t>& file);

VoidFuture close(const std::unique_ptr<file_t>& file);

IntFuture size(const std::unique_ptr<file_t>& file);

} // file
} // seastar_ffi

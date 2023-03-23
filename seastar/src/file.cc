#include "file.hh"
#include "seastar/src/file.rs.h"

namespace seastar_ffi {
namespace file {

open_flags parse_options(const OpenOptions& opts) {
    bool read = opts.get_read();
    bool write = opts.get_write();
    bool create = opts.get_create();

    open_flags flags = open_flags(0);

    if (read) flags |= open_flags::ro;
    if (write) flags |= open_flags::wo;
    if (create) flags |= open_flags::create;

    return flags;
}

VoidFuture open_dma(std::unique_ptr<file_t>& file, rust::str name, const OpenOptions& opts) {
    std::string_view sv_name(name.begin(), name.size());
    open_flags flags = parse_options(opts);
    file_t new_file = co_await seastar::open_file_dma(sv_name, flags);
    file = std::make_unique<file_t>(std::move(new_file));
}

IntFuture read_dma(const std::unique_ptr<file_t>& file, uint8_t* buffer, uint64_t size, uint64_t pos) {
    co_return co_await file->dma_read(pos, buffer, size);
}

IntFuture write_dma(const std::unique_ptr<file_t>& file, uint8_t* buffer, uint64_t size, uint64_t pos) {
    co_return co_await file->dma_write(pos, buffer, size);
}

VoidFuture flush(const std::unique_ptr<file_t>& file) {
    co_await file->flush();
}

VoidFuture close(const std::unique_ptr<file_t>& file) {
    co_await file->close();
}

IntFuture size(const std::unique_ptr<file_t>& file) {
    co_return co_await file->size();
}

} // file
} // seastar_ffi

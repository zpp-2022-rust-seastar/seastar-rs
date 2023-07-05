#pragma once

#include <memory>
#include <cstdint>
#include <seastar/util/log.hh>

#include "rust/cxx.h"

namespace seastar_ffi {
namespace logger {

using logger = seastar::logger;
struct FormatCtx;

std::unique_ptr<logger> new_logger(rust::Str name);
void log(const logger& l, uint32_t level, const FormatCtx& ctx) noexcept;

struct log_writer {
    seastar::internal::log_buf::inserter_iterator it;
    void write(rust::Slice<const uint8_t> data) noexcept;
};

}
}

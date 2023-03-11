#include <algorithm>

#include "logger.hh"
#include "seastar/src/logger.rs.h"

namespace seastar_ffi {
namespace logger {

std::unique_ptr<logger> new_logger(rust::Str name) {
    auto sname = seastar::sstring(name.data(), name.size());
    return std::make_unique<logger>(std::move(sname));
}

void log(const logger& l, uint32_t level, const FormatCtx& ctx) noexcept {
    seastar::logger::lambda_log_writer writer_wrapper([&] (auto it) {
        log_writer writer{std::move(it)};
        write_log_line(writer, ctx);
        return std::move(writer.it);
    });
    const_cast<logger&>(l).log((seastar::log_level)level, writer_wrapper);
}

void log_writer::write(rust::Slice<const uint8_t> data) noexcept {
    it = std::copy(data.begin(), data.end(), std::move(it));
}

}
}

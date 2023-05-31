#pragma once

#include "rust/cxx.h"
#include "cxx-async/include/rust/cxx_async.h"
#include "cxx-async/include/rust/cxx_async_seastar.h"
#include <seastar/net/api.hh>
#include <seastar/core/seastar.hh>
#include <seastar/core/coroutine.hh>

CXXASYNC_DEFINE_FUTURE(void, net_ffi, VoidFuture);
CXXASYNC_DEFINE_FUTURE(rust::String, net_ffi, StringFuture);

namespace net_ffi {

using seastar::server_socket;
using seastar::connected_socket;
using input_stream = seastar::input_stream<char>;
using output_stream = seastar::output_stream<char>;

std::unique_ptr<server_socket> listen(uint16_t port);

VoidFuture accept(
    const std::unique_ptr<server_socket>& server_socket,
    std::unique_ptr<connected_socket>& socket
);

std::unique_ptr<input_stream> get_input_stream(
    const std::unique_ptr<connected_socket>& socket
);

std::unique_ptr<output_stream> get_output_stream(
    const std::unique_ptr<connected_socket>& socket
);

VoidFuture close_output_stream(const std::unique_ptr<output_stream>& output);

StringFuture read(const std::unique_ptr<input_stream>& input);

VoidFuture write(
    const std::unique_ptr<output_stream>& output,
    const rust::str msg
);

VoidFuture flush_output(const std::unique_ptr<output_stream>& output);

} // namespace net_ffi

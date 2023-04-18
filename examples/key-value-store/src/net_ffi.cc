#include "net_ffi.hh"

namespace net_ffi {

std::unique_ptr<server_socket> listen(uint16_t port) {
    seastar::socket_address address(0, port);

    seastar::listen_options options;
    options.proto = seastar::transport::TCP;
    options.reuse_address = true;

    server_socket socket = seastar::listen(address, options);
    return std::make_unique<server_socket>(std::move(socket));
}

VoidFuture accept(
    const std::unique_ptr<server_socket>& server_socket,
    std::unique_ptr<connected_socket>& socket
) {
    seastar::accept_result result = co_await server_socket->accept();
    socket = std::make_unique<connected_socket>(std::move(result.connection));
}

std::unique_ptr<input_stream> get_input_stream(
    const std::unique_ptr<connected_socket>& socket
) {
    input_stream input = socket->input();
    return std::make_unique<input_stream>(std::move(input));
}

std::unique_ptr<output_stream> get_output_stream(
    const std::unique_ptr<connected_socket>& socket
) {
    output_stream output = socket->output();
    return std::make_unique<output_stream>(std::move(output));
}

VoidFuture close_output_stream(const std::unique_ptr<output_stream>& output) {
    co_await output->close();
}

StringFuture read(const std::unique_ptr<input_stream>& input) {
    auto buffer = co_await input->read();
    co_return rust::String(buffer.begin(), buffer.size());
}

VoidFuture write(
    const std::unique_ptr<output_stream>& output,
    const rust::str msg
) {
    co_await output->write(seastar::sstring(msg.begin(), msg.size()));
}

VoidFuture flush_output(const std::unique_ptr<output_stream>& output) {
    co_await output->flush();
}

} // namespace net_ffi

#include "gate.hh"

namespace seastar_ffi {
namespace gate {

std::unique_ptr<gate> new_gate() {
    return std::make_unique<gate>();
}

std::unique_ptr<gate_holder> new_gate_holder(const std::unique_ptr<gate>& gate) {
    return std::make_unique<gate_holder>(*gate);
}

VoidFuture close_gate(const std::unique_ptr<gate>& gate) {
    co_await gate->close();
}

} // namespace gate
} // namespace seastar_ffi

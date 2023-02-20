#include "gate.hh"

namespace seastar_ffi {
namespace gate {

std::unique_ptr<gate> new_gate() {
    return std::make_unique<gate>();
}

} // namespace gate
} // namespace seastar_ffi

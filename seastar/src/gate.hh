#pragma once

#include <seastar/core/gate.hh>

namespace seastar_ffi {
namespace gate {

using gate = seastar::gate;

std::unique_ptr<gate> new_gate();

} // namespace gate
} // namespace seastar_ffi

#pragma once

#include <seastar/core/gate.hh>

namespace seastar_ffi {
namespace gate {

using gate = seastar::gate;
using gate_holder = gate::holder;

std::unique_ptr<gate> new_gate();

std::unique_ptr<gate_holder> new_gate_holder(const std::unique_ptr<gate>& gate);

} // namespace gate
} // namespace seastar_ffi

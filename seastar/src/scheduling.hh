#pragma once

#include "cxx_async_futures.hh"
#include <seastar/core/scheduling.hh>

namespace seastar_ffi {
namespace scheduling {

using scheduling_group = seastar::scheduling_group;

std::shared_ptr<scheduling_group> new_sg();

} // namespace scheduling
} // namespace seastar_ffi

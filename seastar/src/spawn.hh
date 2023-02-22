#pragma once

#include "cxx-async/include/rust/cxx_async_seastar.h"
#include "rust/cxx.h"
#include "cxx_async_futures.hh"

namespace seastar_ffi {
namespace spawn {

VoidFuture cpp_spawn(VoidFuture future);

} // namespace spawn
} // namespace seastar_ffi

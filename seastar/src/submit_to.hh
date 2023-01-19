#pragma once

#include "cxx-async/include/rust/cxx_async_seastar.h"
#include "rust/cxx.h"
#include "cxx_async_futures.hh"

namespace seastar_ffi {

namespace submit_to {

using fn_once_caller_t = VoidFuture (*)(const uint8_t*);

VoidFuture submit_to(const uint32_t shard_id, uint8_t* closure, rust::Fn<VoidFuture(uint8_t*)> caller);

} // submit_to

} // seastar_ffi

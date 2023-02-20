#pragma once

#include "rust/cxx.h"
#include "cxx-async/include/rust/cxx_async.h"
#include "cxx-async/include/rust/cxx_async_seastar.h"
#include <seastar/core/coroutine.hh>

CXXASYNC_DEFINE_FUTURE(void, seastar_ffi, VoidFuture);
CXXASYNC_DEFINE_FUTURE(int, seastar_ffi, IntFuture);

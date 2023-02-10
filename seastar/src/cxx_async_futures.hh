#include "rust/cxx.h"
#include "cxx-async/include/rust/cxx_async.h"
#include "cxx-async/include/rust/cxx_async_seastar.h"
#include <seastar/core/coroutine.hh>

CXXASYNC_DEFINE_FUTURE(void, seastar_ffi, config_and_start_seastar, VoidFuture);
CXXASYNC_DEFINE_FUTURE(int, seastar_ffi, config_and_start_seastar, IntFuture);

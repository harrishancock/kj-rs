#pragma once

#include <kj-rs/promise.h>

namespace kj_rs::tests {

kj::Promise<void> new_ready_promise_void();
kj::Promise<void> new_pending_promise_void();
kj::Promise<void> new_coroutine_promise_void();

kj::Promise<void> new_errored_promise_void();
kj::Promise<int32_t> new_ready_promise_i32(int32_t);

}  // namespace kj_rs::tests

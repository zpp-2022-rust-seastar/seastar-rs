#pragma once

#include "cxx_async_futures.hh"
#include <seastar/core/app-template.hh>

namespace seastar_ffi {
namespace config_and_start_seastar {

using app_template = seastar::app_template;
using seastar_options = app_template::seastar_options;

std::unique_ptr<seastar_options> new_options();

rust::Str get_name(const seastar_options& opts);

rust::Str get_description(const seastar_options& opts);

uint32_t get_smp(const seastar_options& opts);

void set_name(seastar_options& opts, const rust::Str name);

void set_description(seastar_options& opts, const rust::Str description);

void set_smp(seastar_options& opts, const uint32_t smp);

std::unique_ptr<app_template> new_app_template_from_options(seastar_options& opts);

int32_t run_void(app_template& app, rust::Slice<const rust::Str> args, VoidFuture fut);

int32_t run_int(app_template& app, rust::Slice<const rust::Str> args, IntFuture fut);

} // namespace config_and_start_seastar
} // namespace seastar

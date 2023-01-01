#pragma once

#include "rust/cxx.h"
#include <seastar/core/app-template.hh>

namespace seastar {

using seastar_options = seastar::app_template::seastar_options;

std::unique_ptr<seastar_options> new_options();

std::string get_name(const std::unique_ptr<const seastar_options>& opts);

std::string get_description(const std::unique_ptr<const seastar_options>& opts);

unsigned get_smp(const std::unique_ptr<const seastar_options>& opts);

void set_name(const std::unique_ptr<seastar_options>& opts, const rust::Str name);

void set_description(const std::unique_ptr<seastar_options>& opts, const rust::Str description);

void set_smp(const std::unique_ptr<seastar_options>& opts, const unsigned smp);

std::unique_ptr<seastar::app_template> new_app_template_from_options(std::unique_ptr<seastar_options>& opts);

//int run_void(std::unique_ptr<seastar::app_template>& app, int ac, int av, rust::Fn<void()> func);
//
//int run_int(std::unique_ptr<seastar::app_template>& app, int ac, int av, rust::Fn<int()> func);

} // namespace seastar

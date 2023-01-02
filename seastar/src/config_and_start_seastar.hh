#pragma once

#include "rust/cxx.h"
#include <seastar/core/app-template.hh>

namespace seastar {

using seastar_options = app_template::seastar_options;

std::unique_ptr<seastar_options> new_options();

const std::string& get_name(const std::unique_ptr<seastar_options>& opts);

const std::string& get_description(const std::unique_ptr<seastar_options>& opts);

uint32_t get_smp(const std::unique_ptr<seastar_options>& opts);

void set_name(std::unique_ptr<seastar_options>& opts, const rust::Str name);

void set_description(std::unique_ptr<seastar_options>& opts, const rust::Str description);

void set_smp(std::unique_ptr<seastar_options>& opts, const uint32_t smp);

std::unique_ptr<seastar::app_template> new_app_template_from_options(const std::unique_ptr<seastar_options>& opts);

//int run_void(std::unique_ptr<seastar::app_template>& app, int ac, int av, rust::Fn<void()> func);
//
//int run_int(std::unique_ptr<seastar::app_template>& app, int ac, int av, rust::Fn<int()> func);

} // namespace seastar

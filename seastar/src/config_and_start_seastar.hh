#pragma once

#include "rust/cxx.h"
#include <seastar/core/app-template.hh>

using seastar_opts = seastar::app_template::seastar_options;

std::unique_ptr<seastar_opts> new_options();

std::string get_name(const std::unique_ptr<const seastar_opts>& opts);

std::string get_description(const std::unique_ptr<const seastar_opts>& opts);

unsigned get_smp(const std::unique_ptr<const seastar_opts>& opts);

void set_name(const std::unique_ptr<seastar_opts>& opts, const rust::Str name);

void set_description(const std::unique_ptr<seastar_opts>& opts, const rust::Str description);

void set_smp(const std::unique_ptr<seastar_opts>& opts, const unsigned smp);

std::unique_ptr<seastar::app_template> new_app_template_from_options(const std::unique_ptr<const seastar_opts>& opts);
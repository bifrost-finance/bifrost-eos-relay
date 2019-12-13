#include <eosio/bridge_plugin/bridge_plugin.hpp>
#include <eosio/chain/exceptions.hpp>
#include <fc/log/logger_config.hpp>

namespace eosio {
   static appbase::abstract_plugin& _bridge_plugin = app().register_plugin<bridge_plugin>();

class bridge_plugin_impl {
   public:
    chain_plugin* chain_plug = nullptr;

};

bridge_plugin::bridge_plugin():my(new bridge_plugin_impl()){}
bridge_plugin::~bridge_plugin(){}

void bridge_plugin::set_program_options(options_description&, options_description& cfg) {
   cfg.add_options()
         ("option-name", bpo::value<string>()->default_value("default value"),
          "Option Description")
         ;
    ilog("bridge_plugin::set_program_options.");
}

void bridge_plugin::plugin_initialize(const variables_map& options) {
    ilog("bridge_plugin::plugin_initializ.");

    try {
      if( options.count( "option-name" )) {
         // Handle the option
      }
      my->chain_plug = app().find_plugin<chain_plugin>();
    }
   FC_LOG_AND_RETHROW()
}

void bridge_plugin::plugin_startup() {
   // Make the magic happen
    ilog("bridge_plugin::plugin_startup.");
}

void bridge_plugin::plugin_shutdown() {
   // OK, that's enough magic
    ilog("bridge_plugin::plugin_shutdown.");
}

}

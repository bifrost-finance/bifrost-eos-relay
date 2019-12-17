#pragma once
#include <appbase/application.hpp>
#include <eosio/chain_plugin/chain_plugin.hpp>
#include <eosio/chain/types.hpp>

namespace eosio {

using namespace appbase;
using namespace chain;

class bridge_plugin : public appbase::plugin<bridge_plugin> {
public:
   bridge_plugin();
   virtual ~bridge_plugin();

   APPBASE_PLUGIN_REQUIRES((chain_plugin))
   virtual void set_program_options(options_description&, options_description& cfg) override;

   void plugin_initialize(const variables_map& options);
   void plugin_startup();
   void plugin_shutdown();

private:
   std::unique_ptr<class bridge_plugin_impl> my;
};

   struct bridge_blocks {
      block_id_type                             id;
      block_state                               bls;
   };

   struct bridge_change_schedule {
      block_id_type                             id;
      incremental_merkle                        imcre_merkle;
      std::vector<signed_block_header>          block_headers;
      std::vector<std::vector<block_id_type>>   block_id_lists;
      uint8_t                                   status;
   };

   struct bridge_prove_action {
      block_id_type                             id;
      action                                    act;
      action_receipt                            act_receipt;
      std::vector<block_id_type>                act_receipt_merkle_paths;
      incremental_merkle                        imcre_merkle;
      std::vector<signed_block_header>          block_headers;
      std::vector<std::vector<block_id_type>>   block_id_lists;
      uint8_t                                   status;
   };

}

FC_REFLECT( eosio::bridge_blocks, (id)(bls) )
FC_REFLECT( eosio::bridge_change_schedule, (id)(imcre_merkle)(block_headers)(block_id_lists)(status) )
FC_REFLECT( eosio::bridge_prove_action, (id)(act)(act_receipt)(act_receipt_merkle_paths)(imcre_merkle)(block_headers)(block_id_lists)(status) )

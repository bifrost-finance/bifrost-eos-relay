#pragma once
#include <appbase/application.hpp>
#include <eosio/chain_plugin/chain_plugin.hpp>
#include <eosio/chain/types.hpp>

namespace eosio {

using namespace appbase;
using namespace chain;

using checksum256 = block_id_type;

struct action_ffi {
   account_name               account;
   action_name                name;
   const permission_level     *authorization;
   size_t                     authorization_size;
   const char                 *data;
   size_t                     data_size;
};

action_ffi convert_ffi(const action &act) {
   action_ffi act_ffi;
   act_ffi.account = act.account;
   act_ffi.name = act.name;
   act_ffi.authorization = act.authorization.data();
   act_ffi.authorization_size = act.authorization.size();
   act_ffi.data = act.data.data();
   act_ffi.data_size = act.data.size();

   return act_ffi;
}

struct block_id_type_list {
   const block_id_type        *id;
   size_t                     ids_size;
   block_id_type_list() {
      id = nullptr;
      ids_size = 0;
   }
};

block_id_type_list convert_ffi(const std::vector<block_id_type> &ids) {
   if (ids.empty()) {
      return block_id_type_list();
   }
   block_id_type_list ids_ffi_ffi;
   ids_ffi_ffi.id = ids.data();
   ids_ffi_ffi.ids_size = ids.size();

   return ids_ffi_ffi;
}

struct incremental_merkle_ffi {
   uint64_t                         _node_count;
   const block_id_type              *_active_nodes;
   size_t                           _active_nodes_size;
};

incremental_merkle_ffi convert_ffi(const incremental_merkle &im) {
   incremental_merkle_ffi im_ffi;
   im_ffi._node_count = im._node_count;
   im_ffi._active_nodes = im._active_nodes.data();
   im_ffi._active_nodes_size = im._active_nodes.size();

   return im_ffi;
}

struct action_receipt_ffi {
   account_name                    receiver;
   digest_type                     act_digest;
   uint64_t                        global_sequence = 0; ///< total number of actions dispatched since genesis
   uint64_t                        recv_sequence   = 0; ///< total number of actions with this receiver since genesis
   const flat_map<account_name,uint64_t> *auth_sequence;
   size_t                          auth_sequence_size;
   fc::unsigned_int                code_sequence = 0; ///< total number of setcodes
   fc::unsigned_int                abi_sequence  = 0;
};

/*
action_receipt_ffi convert_ffi(const action_receipt& act_receipt) {
   action_receipt_ffi receipt_ffi;
   receipt_ffi.receiver = act_receipt.receiver;
   receipt_ffi.act_digest = act_receipt.act_digest;
   receipt_ffi.global_sequence = act_receipt.global_sequence;
   receipt_ffi.recv_sequence = act_receipt.recv_sequence;
   receipt_ffi.auth_sequence = &act_receipt.auth_sequence.cbegin(); // error: taking the address of a temporary object of type 'boost::container::flat_map<eosio::chain::name, unsigned long long,
   receipt_ffi.auth_sequence_size = act_receipt.auth_sequence.size();
   receipt_ffi.code_sequence = act_receipt.code_sequence;
   receipt_ffi.abi_sequence = act_receipt.abi_sequence;

   return receipt_ffi;
}
*/

struct producer_schedule_type_ffi {
   uint32_t                                       version = 0; ///< sequentially incrementing version number
   const producer_key                             *producers;
   size_t                                         producers_size;
};

struct signed_block_header_ffi {
   block_timestamp_type             timestamp;
   account_name                     producer;
   uint16_t                         confirmed = 1;
   block_id_type                    previous;
   checksum256_type                 transaction_mroot;
   checksum256_type                 action_mroot;
   uint32_t                         schedule_version = 0;
   producer_schedule_type_ffi       *new_producers;
   extensions_type                  header_extensions;

   const char*                            producer_signature;
};

producer_schedule_type_ffi convert_ffi(const producer_schedule_type &ps) {
   producer_schedule_type_ffi ps_ffi;
   ps_ffi.version = ps.version;
   ps_ffi.producers = ps.producers.data();
   ps_ffi.producers_size = ps.producers.size();

   return ps_ffi;
}

struct producer_key_ffi {

};

signed_block_header_ffi convert_ffi(const signed_block_header &header) {
   signed_block_header_ffi header_ffi;
   header_ffi.timestamp = header.timestamp;
   header_ffi.producer = header.producer;
   header_ffi.confirmed = header.confirmed;
   header_ffi.previous = header.previous;
   header_ffi.transaction_mroot = header.transaction_mroot;
   header_ffi.action_mroot = header.action_mroot;
   header_ffi.schedule_version = header.schedule_version;

   if (header.new_producers) {
      header_ffi.new_producers = nullptr;
   } else {
      header_ffi.new_producers = nullptr;
   }

   std::string sig = static_cast<std::string>(header.producer_signature);
   header_ffi.producer_signature = sig.data();

   return header_ffi;
}

struct rpc_result {
   bool success;
   char* msg;
};

}

FC_REFLECT( eosio::action_ffi, (account)(name)(authorization)(authorization_size)(data)(data_size) )

#pragma once
#include <appbase/application.hpp>
#include <eosio/chain_plugin/chain_plugin.hpp>
#include <eosio/chain/types.hpp>

namespace eosio {

using namespace appbase;
using namespace chain;

using checksum256 = block_id_type;

struct rpc_result {
   bool success;
   char* msg;
};

struct action_ffi {
   account_name                     account;
   action_name                      name;
   const permission_level           *authorization;
   size_t                           authorization_size;
   const char                       *data;
   size_t                           data_size;
};

struct block_id_type_list {
   const block_id_type              *id;
   size_t                           ids_size;
   block_id_type_list() {
      id = nullptr;
      ids_size = 0;
   }
};

struct incremental_merkle_ffi {
   uint64_t                         _node_count;
   const block_id_type              *_active_nodes;
   size_t                           _active_nodes_size;
};

struct flat_map_ffi {
   const std::pair<account_name,uint64_t> *auth_sequence;
   size_t                          auth_sequence_size;
};

struct action_receipt_ffi {
   account_name                    receiver;
   digest_type                     act_digest;
   uint64_t                        global_sequence = 0;
   uint64_t                        recv_sequence   = 0;
   const std::pair<account_name,uint64_t> *auth_sequence;
   size_t                          auth_sequence_size;
   fc::unsigned_int                code_sequence = 0;
   fc::unsigned_int                abi_sequence  = 0;
};

struct extension {
   uint16_t                        _type;
   const char                      *data;
   size_t                          data_size;
};

struct extensions_type_ffi {
   extension                       *extensions;
   size_t                          extensions_size;
   extensions_type_ffi() {
      extensions_size = 0;
      extensions = nullptr;
   }
   ~extensions_type_ffi() {
      if (extensions) delete []extensions;
   }
};

struct producer_key_ffi {
   account_name                    producer_name;
   const char                      *block_signing_key;
};

struct producer_schedule_type_ffi {
   uint32_t                        version = 0;
   producer_key_ffi                *producers;
   size_t                          producers_size = 0;
   ~producer_schedule_type_ffi() {
      if (producers) delete []producers;
   }
};

struct block_header_ffi {
   block_timestamp_type             timestamp;
   account_name                     producer;
   uint16_t                         confirmed = 1;
   const char                       *previous;
   const char                       *transaction_mroot;
   const char                       *action_mroot;
   uint32_t                         schedule_version = 0;
   producer_schedule_type_ffi       *new_producers;
   extensions_type_ffi              *header_extensions;
//   block_header_ffi();
   ~block_header_ffi() {
      if (new_producers) delete new_producers;
      if (header_extensions) delete []header_extensions;
   }
};

struct signed_block_header_ffi {
   block_header_ffi                 *block_header;
   char                             *producer_signature;
   signed_block_header_ffi() {
      block_header = nullptr;
      producer_signature = nullptr;
   }
   signed_block_header_ffi(const signed_block_header &);
   ~signed_block_header_ffi() {
      if (block_header) delete block_header;
      if (producer_signature) delete []producer_signature;
   }
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

block_id_type_list convert_ffi(const std::vector<block_id_type> &ids) {
   if (ids.empty()) {
      return block_id_type_list();
   }
   block_id_type_list ids_ffi_ffi;
   ids_ffi_ffi.id = ids.data();
   ids_ffi_ffi.ids_size = ids.size();

   return ids_ffi_ffi;
}

action_receipt_ffi convert_ffi(const action_receipt& act_receipt) {
   action_receipt_ffi receipt_ffi;
   receipt_ffi.receiver = act_receipt.receiver;
   receipt_ffi.act_digest = act_receipt.act_digest;
   receipt_ffi.global_sequence = act_receipt.global_sequence;
   receipt_ffi.recv_sequence = act_receipt.recv_sequence;

   receipt_ffi.auth_sequence = &*act_receipt.auth_sequence.cbegin();
   receipt_ffi.auth_sequence_size = act_receipt.auth_sequence.size();

   receipt_ffi.code_sequence = act_receipt.code_sequence;
   receipt_ffi.abi_sequence = act_receipt.abi_sequence;

   return receipt_ffi;
}

producer_key_ffi convert_ffi(const producer_key &pk) {
   producer_key_ffi key_ffi;
   key_ffi.producer_name = pk.producer_name;

   std::string sig = static_cast<std::string>(pk.block_signing_key);
   key_ffi.block_signing_key = sig.c_str();

   return key_ffi;
}

producer_schedule_type_ffi convert_ffi(const producer_schedule_type &ps) {
   producer_schedule_type_ffi ps_ffi;
   ps_ffi.version = ps.version;
   ps_ffi.producers_size = ps.producers.size();
   if (ps_ffi.producers_size == 0) {
      ps_ffi.producers = nullptr;
   } else {
      ps_ffi.producers = new producer_key_ffi[ps_ffi.producers_size];
      for (size_t i = 0; i < ps_ffi.producers_size; ++i) {
         producer_key_ffi p = convert_ffi(ps.producers[i]);
         ps_ffi.producers[i].producer_name = p.producer_name;
         ps_ffi.producers[i].block_signing_key = p.block_signing_key;
      }
   }

   return ps_ffi;
}

extensions_type_ffi convert_ffi(const extensions_type &ext) {
   auto len = ext.size();
   if (len == 0) return extensions_type_ffi();

   extensions_type_ffi ext_ffi;
   ext_ffi.extensions_size = len;
   ext_ffi.extensions = new extension[len];
   for (size_t i = 0; i < len; ++i) {
      auto e = std::get<1>(ext[i]);
      ext_ffi.extensions[i] = extension {
         std::get<0>(ext[i]),
         e.data(),
         e.size(),
      };
   }

   return ext_ffi;
}

incremental_merkle_ffi convert_ffi(const incremental_merkle &im) {
   incremental_merkle_ffi im_ffi;
   im_ffi._node_count = im._node_count;
   im_ffi._active_nodes = im._active_nodes.data();
   im_ffi._active_nodes_size = im._active_nodes.size();

   return im_ffi;
}

signed_block_header_ffi::signed_block_header_ffi(const signed_block_header &header) {
   block_header_ffi header_ffi;
   header_ffi.timestamp = header.timestamp;
   header_ffi.producer = header.producer;
   header_ffi.confirmed = header.confirmed;
   header_ffi.previous = header.previous.data();
   header_ffi.transaction_mroot = header.transaction_mroot.data();
   header_ffi.action_mroot = header.action_mroot.data();
   header_ffi.schedule_version = header.schedule_version;

   if (header.new_producers) {
      auto h = convert_ffi(*(header.new_producers));
      header_ffi.new_producers->version = h.version;
      header_ffi.new_producers->producers = h.producers;
      header_ffi.new_producers->producers_size = h.producers_size;
   } else {
      header_ffi.new_producers = nullptr;
   }

   auto e = convert_ffi(header.header_extensions);
   if (e.extensions_size == 0) {
      header_ffi.header_extensions = nullptr;
   } else {
      header_ffi.header_extensions = new extensions_type_ffi[e.extensions_size];
      header_ffi.header_extensions->extensions = e.extensions;
      header_ffi.header_extensions->extensions_size = e.extensions_size;
   }

   std::string sig = static_cast<std::string>(header.producer_signature);
   producer_signature = new char[sig.size() + 1];
   strcpy(producer_signature, sig.c_str());

   block_header = new block_header_ffi();
   memcpy(block_header, &header_ffi, sizeof(header_ffi));
}

}

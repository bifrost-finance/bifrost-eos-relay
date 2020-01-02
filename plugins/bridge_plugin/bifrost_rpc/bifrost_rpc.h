// Copyright 2019 Liebi Technologies.
// This file is part of Bifrost.

// Bifrost is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Bifrost is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Bifrost.  If not, see <http://www.gnu.org/licenses/>.

#ifndef BIFROST_RPC
#define BIFROST_RPC

#include <string>
#include <optional>
#include <utility>

#include <eosio/bridge_plugin/ffi_types.hpp>
#include <eosio/chain_plugin/chain_plugin.hpp>
#include <eosio/chain/types.hpp>

#ifdef __cplusplus
extern "C" {
#endif

// bifrost rpc api

void change_schedule(
   char* url,
   char* signer,
   const eosio::incremental_merkle_ffi* imcre_merkle,
   const eosio::signed_block_header_ffi *blocks_ffi,
   size_t blocks_ffi_size,
   const eosio::block_id_type_list* ids_list,
   size_t ids_list_size
);

eosio::rpc_result *prove_action(
   const char* url,
   const char* signer,
   const eosio::action_ffi* act_ffi,
   const eosio::incremental_merkle_ffi* imcre_merkle,
   const eosio::action_receipt_ffi* act_receipt,
   const eosio::block_id_type_list *action_merkle_paths,
   const eosio::signed_block_header_ffi *blocks_ffi,
   size_t blocks_ffi_size,
   const eosio::block_id_type_list* ids_list,
   size_t ids_list_size
);

#ifdef __cplusplus
}
#endif

#endif /* BIFROST_RPC */

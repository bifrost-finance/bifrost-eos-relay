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
#include <eosio/chain_plugin/chain_plugin.hpp>
#include <eosio/chain/types.hpp>

#ifdef __cplusplus
extern "C" {
#endif

// bifrost rpc api

void change_schedule(
        char* url,
        char* signer,
        const eosio::incremental_merkle* merkle,
        size_t merkle_checksum_len,
        const eosio::signed_block_header *block_headers,
        size_t block_headers_len,
        const std::vector<std::vector<eosio::block_id_type>> block_ids_list
);

bool prove_action(
        const char* url,
        const char* signer,
        const char* action_json,
        const char* receipt_json,
        const eosio::block_id_type *action_merkle_paths,
        size_t action_merkle_paths_len,
        const eosio::block_id_type* _active_nodes,
        size_t merkle_checksum_len,
        uint64_t _node_count,
        const char* blocks_json,
        const char* ids_json
);

#ifdef __cplusplus
}
#endif

#endif /* BIFROST_RPC */

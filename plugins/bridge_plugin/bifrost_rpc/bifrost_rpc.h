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
extern "C" { // Todo, after include related headers, extern C++ support template, use extern "C"
#endif

// bifrost rpc api

void change_schedule(
        char* url,
        char* signer,
        const eosio::incremental_merkle* merkle,
        size_t merkle_checksum_len,
        const std::vector<eosio::signed_block_header> block_headers,
        size_t block_headers_len,
        const std::vector<std::vector<eosio::block_id_type>> block_ids_list
);

void prove_action(
        char* url,
        char* signer,
        const eosio::action* action,
        size_t action_auth_len,
        size_t action_data_len,
        const eosio::action_receipt* action_receipt,
        size_t auth_sequence_len,
        const std::vector<eosio::block_id_type> action_merkle_paths,
        size_t action_merkle_paths_len,
        const eosio::incremental_merkle* merkle,
        size_t merkle_checksum_len,
        const std::vector<eosio::signed_block_header> block_headers,
        size_t block_headers_len,
        const std::vector<std::vector<eosio::block_id_type>> block_ids_list
);

#ifdef __cplusplus
}
#endif

#endif /* BIFROST_RPC */

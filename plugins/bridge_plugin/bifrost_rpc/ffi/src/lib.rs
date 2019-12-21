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

use eos_chain::Checksum256;
use log::info;
use keyring::AccountKeyring;
use primitives::crypto::Pair;
use rpc_client::{
    Api,
    compose_call, compose_extrinsic,
    extrinsic::xt_primitives::UncheckedExtrinsicV4,
};
use std::{
    os::raw::c_char,
    slice,
};

mod ffi_types;
use ffi_types::*;

#[no_mangle]
pub extern fn change_schedule(
    url: *const c_char,
    signer: *const c_char,
    merkle: *const IncrementalMerkleFFI,
    merkle_checksum_len: size_t,
    block_headers: *const SignedBlockHeaderFFI, // vec<>
    block_headers_len: size_t,
    block_ids_list: *const *const Checksum256 // vec<vec<>>
) {
    // check pointers null or not
    // Todo, find a more elegant way to check these pointers null or not
    match (url.is_null(), signer.is_null(), merkle.is_null(), block_headers.is_null(), block_ids_list.is_null()) {
        (true, true, true, true, true) => {
            info!("all are valid pointers.");
        }
        _ => {
            return;
        }
    }
    let url = unsafe {
        cstr_to_string(url).expect("failed to convert cstring to rust string.") // Todo, remove expect
    };
    let signer = AccountKeyring::Alice.pair();
    let api = Api::new(format!("ws://{}", url)).set_signer(signer.clone());

    let merkle = unsafe {
        (*merkle).clone().into_incrementl_merkle(merkle_checksum_len)
    };

    let block_headers = unsafe {
        let ffi = slice::from_raw_parts(block_headers, block_headers_len);
        ffi.into_iter().map(|f| f.into_signed_block_header()).collect::<Vec<_>>()
    };

    let block_ids_list = unsafe {
        let ffi = slice::from_raw_parts(block_ids_list, 15).to_vec();
        ffi.into_iter().map(|f|{
//            slice::from_raw_parts(f, 11).iter().map(|c| ptr::read(c)).collect::<Vec<Checksum256>>()
            slice::from_raw_parts(f, 10).to_vec()
        }).collect::<Vec<_>>()
    };
//    let block_ids_list: Vec<Vec<Checksum256>> = Vec::new();

    let proposal = compose_call!(
        api.metadata.clone(),
        "BridgeEOS",
        "change_schedule",
        merkle,
        block_headers,
        block_ids_list
    );

    let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
        api.clone(),
        "Sudo",
        "sudo",
        proposal
    );

    // Unable to decode Vec on index 2 createType(ExtrinsicV4):: Source is too large
    println!("[+] Composed extrinsic: {:?}\n", xt);
    // send and watch extrinsic until finalized
    let tx_hash = api.send_extrinsic(xt.hex_encode()).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);
}

#[no_mangle]
pub extern fn prove_action(
    url: *const c_char,
    signer: *const c_char,
    action: *const ActionFFI,
    action_auth_len: size_t,
    action_data_len: size_t,
    action_receipt: *const ActionReceiptFFI,
    auth_sequence_len: size_t,
    action_merkle_paths: *const Checksum256,
    action_merkle_paths_len: size_t,
    merkle: *const IncrementalMerkleFFI,
    merkle_checksum_len: size_t,
    block_headers: *const SignedBlockHeaderFFI,
    block_headers_len: size_t,
    block_ids_list: *const *const Checksum256
) {
    // check the params pointers are null or not
    match (
        url.is_null(), signer.is_null(), action.is_null(), action_receipt.is_null(), action_merkle_paths.is_null(),
        merkle.is_null(), block_headers.is_null(), block_ids_list.is_null()
    ) {
        (true, true, true, true, true, true, true, true) => {
            // normal code
            info!("all are valid pointers.");
        }
        _ => {
            // there's a null pointer among params
            // exception happends.
            return;
        }
    }

    let action = unsafe {
        (*action).clone().into_action(action_auth_len, action_data_len).expect("failed to read value from c++ pointer.") // Todo, remove expect
    };

    let action_receipt = unsafe {
        (*action_receipt).clone().into_action_receipt(auth_sequence_len).expect("failed to read value from c++ pointer.") // Todo, remove expect
    };

    let action_merkle_paths = unsafe {
        slice::from_raw_parts(action_merkle_paths, action_merkle_paths_len).to_vec()
    };

    let merkle = unsafe {
        (*merkle).clone().into_incrementl_merkle(merkle_checksum_len)
    };

    let block_headers = unsafe {
        slice::from_raw_parts(block_headers, block_headers_len).iter().map(|f| f.into_signed_block_header()).collect::<Vec<_>>()
    };

    let block_ids_list = unsafe {
        let ffi = slice::from_raw_parts(block_ids_list, 15).to_vec();
        ffi.into_iter().map(|f| {
            slice::from_raw_parts(f, 10).to_vec()
        }).collect::<Vec<_>>()
    };

    let url = unsafe {
        cstr_to_string(url).expect("failed to convert cstring to rust string.")
    };
    let signer = AccountKeyring::Alice.pair();
    let api = Api::new(format!("ws://{}", url)).set_signer(signer.clone());

    let proposal = compose_call!(
        api.metadata.clone(),
        "BridgeEOS",
        "prove_action",
        action,
        action_receipt,
        action_merkle_paths,
        merkle,
        block_headers,
        block_ids_list
    );

    let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
        api.clone(),
        "Sudo",
        "sudo",
        proposal
    );

    println!("[+] Composed extrinsic: {:?}\n", xt);
    // send and watch extrinsic until finalized
    let tx_hash = api.send_extrinsic(xt.hex_encode()).unwrap();
    println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);
}

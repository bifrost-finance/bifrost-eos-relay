// Copyright 2019-2020 Liebi Technologies.
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

use eos_chain::{Action, ActionReceipt, Checksum256, IncrementalMerkle, ProducerAuthoritySchedule, SignedBlockHeader};
use std::{
    convert::TryInto,
    fmt::{self, Display},
    os::raw::c_char,
    ptr,
    slice,
};

mod ffi_types;
use ffi_types::*;

mod ffi_futures;
use ffi_futures::*;

mod rpc_calls;

#[derive(Clone, Debug)]
pub enum Error {
    NullPtr(String),
    CStrConvertError,
    PublicKeyError,
    SignatureError,
    WrongSudoSeed,
    SubxtError(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NullPtr(ref who_is_null) => write!(f, "{} is null pointer.", who_is_null),
            Self::CStrConvertError => write!(f, "Failed to convert c string to rust string."),
            Self::PublicKeyError => write!(f, "Failed to convert string to PublicKey."),
            Self::SignatureError => write!(f, "Failed to convert string to Signature."),
            Self::WrongSudoSeed => write!(f, "Wrong sudo seed, failed to sign transaction."),
            Self::SubxtError(e) => write!(f, "Error from subxt crate: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Self::NullPtr(_) => "Null pointer.",
            Self::CStrConvertError => "Failed to convert c string to rust string.",
            Self::PublicKeyError => "Failed to convert string to PublicKeyError.",
            Self::SignatureError => "Failed to convert string to Signature.",
            Self::WrongSudoSeed => "Wrong sudo seed, failed to sign transaction.",
            Self::SubxtError(e) => e,
        }
    }
}

#[no_mangle]
pub extern "C" fn change_schedule(
    url:                  *const c_char,
    signer:               *const c_char,
    legacy_schedule_hash: Checksum256,
    schedule:             *const ProducerAuthorityScheduleFFI,
    imcre_merkle:         *const IncrementalMerkleFFI,
    blocks_ffi:           *const SignedBlockHeaderFFI,
    blocks_ffi_size:      size_t,
    ids_list:             *const Checksum256FFI,
    ids_list_size:        size_t
) -> Box<RpcResponse> {
    // check pointers null or not
    match (url.is_null(), signer.is_null(), schedule.is_null(), imcre_merkle.is_null(), blocks_ffi.is_null(), ids_list.is_null()) {
        (false, false, false, false, false, false) => (),
        _ => {
            return generate_raw_result(false, "cannot send action to bifrost node to prove it due to there're null points");
        }
    }

    let url = {
        let url = char_to_string(url);
        if url.is_err() {
            return generate_raw_result(false, "This is not an valid bifrost node address.");
        }
        url.unwrap()
    };

    let signer = {
        let signer = char_to_string(signer);
        if signer.is_err() {
            return generate_raw_result(false, "This is not an valid bifrost node address.");
        }
        signer.unwrap()
    };

    let new_schedule = {
        let schedule_ffi = &unsafe { ptr::read(schedule) };
        let new_schedule: Result<ProducerAuthoritySchedule, _> = schedule_ffi.try_into();
        if new_schedule.is_err() {
            return generate_raw_result(false, new_schedule.unwrap_err().to_string());
        }
        new_schedule.unwrap()
    };

    let merkle: IncrementalMerkle = {
        let imcre_merkle = &unsafe { ptr::read(imcre_merkle) };
        let r: Result<IncrementalMerkle, _> = imcre_merkle.try_into();
        if r.is_err() {
            return generate_raw_result(false, r.unwrap_err().to_string());
        }
        r.unwrap()
    };

    let block_headers: Vec<SignedBlockHeader> = {
        let blocks_ffi = &unsafe { slice::from_raw_parts(blocks_ffi, blocks_ffi_size) };
        let mut block_headers: Vec<_> = Vec::with_capacity(blocks_ffi_size);
        for block in blocks_ffi.iter() {
            let ffi = &unsafe { ptr::read(block) };
            let r: Result<SignedBlockHeader, Error> = ffi.try_into();
            if r.is_err() {
                return generate_raw_result(false, r.unwrap_err().to_string());
            }
            block_headers.push(r.unwrap());
        }
        block_headers
    };

    let mut ids_lists: Vec<Vec<Checksum256>>= Vec::with_capacity(15);
    ids_lists.push(Vec::new());
    let ids_list_ffi = &unsafe { slice::from_raw_parts(ids_list, ids_list_size) };
    for ids in ids_list_ffi.iter().skip(1) { // skip first ids due to it's am empty list(null pointer)
        let r: Result<Vec<Checksum256>, _> = ids.try_into();
        if r.is_err() {
            return generate_raw_result(false, r.unwrap_err().to_string());
        }
        ids_lists.push(r.unwrap());
    }

    let result = futures::executor::block_on(async move {
        crate::rpc_calls::change_schedule_call(
            url,
            signer,
            legacy_schedule_hash,
            new_schedule,
            merkle,
            block_headers,
            ids_lists,
        ).await
    });

    // send and watch extrinsic until finalized
    match result {
        Ok(tx_hash) => {
            println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);
            generate_raw_result(true, tx_hash.to_string())
        }
        Err(e) => {
            println!("[+] Transaction got failure due to: {:?}\n", e);
            generate_raw_result(false, e.to_string())
        }
    }
}

#[no_mangle]
pub extern "C" fn prove_action(
    url:                 *const c_char,
    signer:              *const c_char,
    act_ffi:             *const ActionFFI,
    imcre_merkle:        *const IncrementalMerkleFFI,
    act_receipt:         *const ActionReceiptFFI,
    action_merkle_paths: *const Checksum256FFI,
    blocks_ffi:          *const SignedBlockHeaderFFI,
    blocks_ffi_size:     size_t,
    ids_list:            *const Checksum256FFI,
    ids_list_size:       size_t
) -> Box<RpcResponse> {
    match (
        url.is_null(), signer.is_null(), act_ffi.is_null(), imcre_merkle.is_null(),
        act_receipt.is_null(), action_merkle_paths.is_null(), blocks_ffi.is_null(), ids_list.is_null()
    ) {
        (false, false, false, false, false, false, false, false) => (),
        _ => { // if there's any null pointer, just return
            return generate_raw_result(false, "cannot send action to bifrost node to prove it due to there're null points");
        }
    }

    let action: Action = {
        let ffi = &unsafe { ptr::read(act_ffi) };
        let r: Result<Action, _> = ffi.try_into();
        if r.is_err() {
            return generate_raw_result(false, r.unwrap_err().to_string());
        }
        r.unwrap()
    };

    let merkle: IncrementalMerkle = {
        let imcre_merkle = &unsafe { ptr::read(imcre_merkle) };
        let r: Result<IncrementalMerkle, _> = imcre_merkle.try_into();
        if r.is_err() {
            return generate_raw_result(false, r.unwrap_err().to_string());
        }
        r.unwrap()
    };

    let action_receipt: ActionReceipt = {
        let act_ffi = &unsafe { ptr::read(act_receipt) };
        let r: Result<ActionReceipt, _> = act_ffi.try_into();
        if r.is_err() {
            return generate_raw_result(false, r.unwrap_err().to_string());
        }
        r.unwrap()
    };

    let action_merkle_paths: Vec<Checksum256> = {
        let paths = &unsafe { ptr::read(action_merkle_paths) };
        let r: Result<Vec<Checksum256>, _> = paths.try_into();
        if r.is_err() {
            return generate_raw_result(false, r.unwrap_err().to_string());
        }
        r.unwrap()
    };

    let block_headers: Vec<SignedBlockHeader> = {
        let blocks_ffi = &unsafe { slice::from_raw_parts(blocks_ffi, blocks_ffi_size) };
        let mut block_headers: Vec<_> = Vec::with_capacity(blocks_ffi_size);
        for block in blocks_ffi.iter() {
            let ffi = &unsafe { ptr::read(block) };
            let r: Result<SignedBlockHeader, Error> = ffi.try_into();
            if r.is_err() {
                return generate_raw_result(false, r.unwrap_err().to_string());
            }
            block_headers.push(r.unwrap());
        }
        block_headers
    };

    let mut ids_lists: Vec<Vec<Checksum256>>= Vec::with_capacity(15);
    ids_lists.push(Vec::new());
    let ids_list_ffi = &unsafe { slice::from_raw_parts(ids_list, ids_list_size) };
    for ids in ids_list_ffi.iter().skip(1) { // skip first ids due to it's am empty list(null pointer)
        let r: Result<Vec<Checksum256>, _> = ids.try_into();
        if r.is_err() {
            return generate_raw_result(false, r.unwrap_err().to_string());
        }
        ids_lists.push(r.unwrap());
    }

    let url = {
        let url = char_to_string(url);
        if url.is_err() {
            return generate_raw_result(false, "This is not an valid bifrost node address.");
        }
        url.unwrap()
    };

    let signer = {
        let signer = char_to_string(signer);
        if signer.is_err() {
            return generate_raw_result(false, "This is not an valid bifrost node address.");
        }
        signer.unwrap()
    };

    let result = futures::executor::block_on(async move {
        crate::rpc_calls::prove_action_call(
            url,
            signer,
            action,
            action_receipt,
            action_merkle_paths,
            merkle,
            block_headers,
            ids_lists,
        ).await
    });

    // send and watch extrinsic until finalized
    match result {
        Ok(tx_hash) => {
            println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);
            generate_raw_result(true, tx_hash.to_string())
        }
        Err(e) => {
            println!("[+] Transaction got failure due to: {:?}\n", e);
            generate_raw_result(false, e.to_string())
        }
    }
}

/*
#[no_mangle]
pub extern "C" fn async_prove_action(
    url:                 *const c_char,
    signer:              *const c_char,
    act_ffi:             *const ActionFFI,
    imcre_merkle:        *const IncrementalMerkleFFI,
    act_receipt:         *const ActionReceiptFFI,
    action_merkle_paths: *const Checksum256FFI,
    blocks_ffi:          *const SignedBlockHeaderFFI,
    blocks_ffi_size:     size_t,
    ids_list:            *const Checksum256FFI,
    ids_list_size:       size_t
) {
    use tokio::runtime::Runtime;
    use tokio::sync::mpsc;
//    match (
//        url.is_null(), signer.is_null(), act_ffi.is_null(), imcre_merkle.is_null(),
//        act_receipt.is_null(), action_merkle_paths.is_null(), blocks_ffi.is_null(), ids_list.is_null()
//    ) {
//        (false, false, false, false, false, false, false, false) => (),
//        _ => { // if there's any null pointer, just return
//            return generate_raw_result(false, "cannot send action to bifrost node to prove it due to there're null points");
//        }
//    }

    // create channel
    let (mut tx, mut rx) = mpsc::channel(6);
    // create runtime handler
    let mut runtime = Runtime::new().unwrap();

    runtime.spawn(async move {
//        let tx = tx.clone();
        let act = ActionFuture { ffi: act_ffi, finished: false}.await;
        tx.send(FuturesData::Action(act)).await;
    });

//    runtime.spawn(async {
//        let tx = tx.clone();
//        let imcre_merkle = IncrementalMerkleFuture { ffi: imcre_merkle, finished: false}.await;
//        tx.send(FuturesData::IncrementalMerkle(imcre_merkle)).await;
//    });

//    runtime.spawn(async {
//        let tx = tx.clone();
//        let act_receipt = ActionReceiptFuture { ffi: act_receipt, finished: false}.await;
//        tx.send(FuturesData::ActionReceipt(act_receipt)).await;
//    });

//    runtime.spawn(async {
//        let tx = tx.clone();
//        let action_merkle_paths = Checksum256Future { ffi: action_merkle_paths, finished: false}.await;
//        tx.send(FuturesData::Checksum256(action_merkle_paths)).await;
//    });

//    runtime.spawn(async {
//        let tx = tx.clone();
//        let headers = SignedBlockHeadersFuture {
//            ffi: blocks_ffi,
//            blocks_ffi_size,
//            finished: false
//        }.await;
//        tx.send(FuturesData::SignedBlockHeader(headers)).await;
//    });

//    runtime.spawn(async {
//        let tx = tx.clone();
//        let ids_list = IdListFuture { ffi: ids_list, ids_list_size, finished: false}.await;
//        tx.send(FuturesData::IdList(ids_list)).await;
//    });

//    let url = {
//        let url = char_to_string(url);
//        if url.is_err() {
//            return generate_raw_result(false, "This is not an valid bifrost node address.");
//        }
//        url.unwrap()
//    };
//    let signer = AccountKeyring::Alice.pair();
//    let api = Api::new(format!("ws://{}", url)).set_signer(signer.clone());
//    let target = AccountKeyring::Alice.public();

    runtime.spawn(async move {

        while let Some(data) = rx.recv().await {
            match data {
                FuturesData::Action(act) => (),
                FuturesData::Checksum256(act) => (),
                FuturesData::ActionReceipt(act) => (),
                FuturesData::IncrementalMerkle(act) => (),
                FuturesData::SignedBlockHeader(act) => (),
                FuturesData::IdList(act) => (),
            }
        }

//        let proposal = compose_call!(
//            api.metadata.clone(),
//            "BridgeEos",
//            "prove_action",
//            target,
//            action,
//            action_receipt,
//            action_merkle_paths,
//            merkle,
//            block_headers,
//            ids_lists
//        );
//
//        let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
//            api.clone(),
//            "Sudo",
//            "sudo",
//            proposal
//        );
//
//        println!("[+] Composed extrinsic: {:?}\n", xt);
//        // send and watch extrinsic until finalized
//        match api.send_extrinsic(xt.hex_encode()) {
//            Ok(tx_hash) => {
//                println!("[+] Transaction got finalized. Hash: {:?}\n", tx_hash);
//                generate_raw_result(true, tx_hash.to_string())
//            }
//            Err(e) => {
//                println!("[+] Transaction got failure due to: {:?}\n", e);
//                generate_raw_result(true, e.to_string())
//            }
//        }
    });

}
*/

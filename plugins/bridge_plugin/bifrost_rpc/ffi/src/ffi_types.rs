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

use eos_chain::{
    Action, AccountName, ActionName, ActionReceipt, PermissionLevel, Checksum256,
    Signature, BlockHeader, Extension, utils::flat_map::FlatMap, UnsignedInt, PublicKey,
    ProducerKey, BlockTimestamp, ProducerSchedule, IncrementalMerkle, SignedBlockHeader
};
use std::os::raw::{c_char, c_uint, c_ushort, c_ulonglong};
use std::{slice, ffi::{CStr, CString}};

#[allow(non_camel_case_types)]
pub(crate) type size_t = usize;
pub(crate) type FFIResult<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send + 'static>>;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ActionFFI {
    pub account: AccountName,
    pub name: ActionName,
    pub authorization: *const PermissionLevel,
    pub data: *const c_char,
}

impl ActionFFI {
    pub(crate) unsafe fn into_action(&self, auth_len: usize, data_len: usize) -> FFIResult<Action> {
        let account = self.account;
        let name = self.name;
        let authorization = slice::from_raw_parts(self.authorization, auth_len).to_vec();
        let data = slice::from_raw_parts(self.data, data_len).iter().map(|c| *c as u8).collect::<Vec<_>>();
        Ok(Action {
            account,
            name,
            authorization,
            data
        })
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ActionReceiptFFI {
    pub receiver: AccountName,
    pub act_digest: Checksum256,
    pub global_sequence: c_ulonglong,
    pub recv_sequence: c_ulonglong,
    pub auth_sequence: FlatMapFFI<AccountName, c_ulonglong>,
    pub code_sequence: UnsignedInt,
    pub abi_sequence: UnsignedInt,
}

impl ActionReceiptFFI {
    pub(crate) unsafe fn into_action_receipt(&self, auth_sequence_len: usize) -> FFIResult<ActionReceipt> {
        let receiver = self.receiver;
        let act_digest = self.act_digest;
        let global_sequence = self.global_sequence;
        let recv_sequence = self.recv_sequence;
        let code_sequence = self.code_sequence.clone();
        let abi_sequence = self.abi_sequence.clone();
        let auth_sequence = self.auth_sequence.into_flat_map(auth_sequence_len); // Todo, do a experiment on c++ class to rust struct
        Ok(ActionReceipt {
            receiver,
            act_digest,
            global_sequence,
            recv_sequence,
            auth_sequence,
            code_sequence,
            abi_sequence
        })
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct FlatMapFFI<K, V> { // Todo
    maps: *const (K, V),
}

impl<K, V> FlatMapFFI<K, V> where K: Clone, V: Clone {
    pub unsafe fn into_flat_map(&self, map_len: usize) -> FlatMap<K, V> {
        let maps = slice::from_raw_parts(self.maps, map_len).to_vec();
        FlatMap::assign(maps)
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct IncrementalMerkleFFI {
    _node_count: c_ulonglong,
    _active_nodes: *const Checksum256,
}

impl IncrementalMerkleFFI {
    pub unsafe fn into_incrementl_merkle(&self, check_sum_len: usize) -> IncrementalMerkle {
        let _node_count = self._node_count;
        let _active_nodes = slice::from_raw_parts(self._active_nodes, check_sum_len).to_vec();
        IncrementalMerkle::new(_node_count, _active_nodes)
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ExtensionFFI(c_ushort, *const c_char);

impl ExtensionFFI {
    pub unsafe fn into_extension(&self, len: usize) -> Extension {
        Extension(self.0, slice::from_raw_parts(self.1, len).iter().map(|c| *c as u8).collect::<Vec<_>>())
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct PublicKeyFFI {
    pub type_: UnsignedInt,
    pub data: *const c_char, // constant array, length is 33
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct ProducerKeyFFI {
    pub producer_name: AccountName,
    pub block_signing_key: PublicKeyFFI,
}


#[derive(Clone, Debug)]
#[repr(C)]
pub struct ProducerScheduleFFI {
    pub version: c_uint,
    pub producers: *const ProducerKey,
//    pub producers: *const ProducerKeyFFI,
}

impl ProducerScheduleFFI {
    pub unsafe fn into_producer_shcedule(&self, producers_count: usize) -> ProducerSchedule {
        ProducerSchedule {
            version: self.version,
            producers: slice::from_raw_parts(self.producers, producers_count).to_vec()
        }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct BlockHeaderFFI {
    pub timestamp: BlockTimestamp,
    pub producer: AccountName,
    pub confirmed: c_ushort,
    pub previous: Checksum256,
    pub transaction_mroot: Checksum256,
    pub action_mroot: Checksum256,
    pub schedule_version: c_uint,
    pub new_producers: *const ProducerScheduleFFI, // Todo, rust Option vs c++ std::optional ?
    pub header_extensions: *const Extension,
}

impl BlockHeaderFFI {
    pub unsafe fn into_block_header(&self, producers_count: usize, extensions_len: Vec<usize>) -> BlockHeader {
        BlockHeader {
            timestamp: self.timestamp,
            producer: self.producer,
            confirmed: self.confirmed,
            previous: self.previous,
            transaction_mroot: self.transaction_mroot,
            action_mroot: self.action_mroot,
            schedule_version: self.schedule_version,
            new_producers: None,
            header_extensions: Default::default()
        }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct SignedBlockHeaderFFI {
    pub block_header: BlockHeaderFFI,
    pub producer_signature: Signature,
}

impl SignedBlockHeaderFFI {
    pub unsafe fn into_signed_block_header(&self) -> SignedBlockHeader {
        let ffi = self.block_header.clone();
        SignedBlockHeader {
            block_header: ffi.into_block_header(1, Vec::new()),
            producer_signature: self.producer_signature.clone()
        }
    }
}

pub(crate) unsafe fn cstr_to_string(cstr: *const c_char) -> FFIResult<String> {
    let cstr = CStr::from_ptr(cstr);
    let rust_string = cstr.to_str()?.to_string();
    Ok(rust_string)
}

pub(crate) fn generate_error(success: bool, err_msg: impl AsRef<str>) -> RustError {
    let c_str = CString::new(err_msg.as_ref())
                .unwrap_or(CString::new("unknow error type.")
                .expect("failed to get raw pointer of error message"));
    RustError {
        success,
        err_msg: c_str.as_ptr()
    }
}

// Todo, this error will return to c++ caller
#[derive(Clone, Debug)]
#[repr(C)]
pub struct RustError {
    success: bool,
    err_msg: *const c_char,
}

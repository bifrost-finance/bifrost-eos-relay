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

use eos_chain::{
    Action, ActionReceipt, Checksum256, IncrementalMerkle, SignedBlockHeader
};
use super::ffi_types::*;
use std::{
    convert::TryInto,
    future::Future,
    task::{Context, Poll},
    pin::Pin,
    ptr,
    slice,
};
use crate::ffi_types::size_t;

pub(crate) enum FuturesData {
    Action(Result<Action, crate::Error>),
    Checksum256(Result<Vec<Checksum256>, crate::Error>),
    ActionReceipt(Result<ActionReceipt, crate::Error>),
    IncrementalMerkle(Result<IncrementalMerkle, crate::Error>),
    SignedBlockHeader(Vec<Result<SignedBlockHeader, crate::Error>>),
    IdList(Vec<Result<Vec<Checksum256>, crate::Error>>),
}

pub(crate) struct ActionFuture {
    ffi: *const ActionFFI,
    finished: bool,
}

impl Future for ActionFuture {
    type Output = Result<Action, crate::Error>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.finished {
            let act: Result<Action, _> = {
                let ffi = &unsafe { ptr::read(self.ffi) };
                ffi.try_into()
            };
            Poll::Ready(act)
        } else {
            cx.waker().wake_by_ref();
            self.get_mut().finished = true;
            Poll::Pending
        }
    }
}

pub(crate) struct Checksum256Future {
    ffi: *const Checksum256FFI,
    finished: bool,
}

impl Future for Checksum256Future {
    type Output = Result<Vec<Checksum256>, crate::Error>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.finished {
            let action_merkle_paths: Result<Vec<Checksum256>, _> = {
                let paths = &unsafe { ptr::read(self.ffi) };
                paths.try_into()
            };
            Poll::Ready(action_merkle_paths)
        } else {
            cx.waker().wake_by_ref();
            self.get_mut().finished = true;
            Poll::Pending
        }
    }
}

pub(crate) struct IdListFuture {
    ffi: *const Checksum256FFI,
    ids_list_size: size_t,
    finished: bool,
}

impl Future for IdListFuture {
    type Output = Vec<Result<Vec<Checksum256>, crate::Error>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.finished {
            let mut ids_lists: Vec<Result<Vec<Checksum256>, crate::Error>> = Vec::with_capacity(15);
            ids_lists.push(Ok(Vec::new()));
            let ids_list_ffi = &unsafe { slice::from_raw_parts(self.ffi, self.ids_list_size) };
            for ids in ids_list_ffi.iter().skip(1) { // skip first ids due to it's am empty list(null pointer)
                let r: Result<Vec<Checksum256>, _> = ids.try_into();
                ids_lists.push(r);
            }

            Poll::Ready(ids_lists)
        } else {
            cx.waker().wake_by_ref();
            self.get_mut().finished = true;
            Poll::Pending
        }
    }
}

pub(crate) struct ActionReceiptFuture {
    ffi: *const ActionReceiptFFI,
    finished: bool,
}

impl Future for ActionReceiptFuture {
    type Output = Result<ActionReceipt, crate::Error>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.finished {
            let action_receipt: Result<ActionReceipt, _> = {
                let receipt_ffi = &unsafe { ptr::read(self.ffi) };
                receipt_ffi.try_into()
            };
            Poll::Ready(action_receipt)
        } else {
            cx.waker().wake_by_ref();
            self.get_mut().finished = true;
            Poll::Pending
        }
    }
}

pub(crate) struct IncrementalMerkleFuture {
    ffi: *const IncrementalMerkleFFI,
    finished: bool,
}

impl Future for IncrementalMerkleFuture {
    type Output = Result<IncrementalMerkle, crate::Error>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.finished {
            let imcre_merkle: Result<IncrementalMerkle, _> = {
                let merkle = &unsafe { ptr::read(self.ffi) };
                merkle.try_into()
            };
            Poll::Ready(imcre_merkle)
        } else {
            cx.waker().wake_by_ref();
            self.get_mut().finished = true;
            Poll::Pending
        }
    }
}

pub(crate) struct SignedBlockHeadersFuture {
    ffi: *const SignedBlockHeaderFFI,
    blocks_ffi_size: size_t,
    finished: bool,
}

impl Future for SignedBlockHeadersFuture {
    type Output = Vec<Result<SignedBlockHeader, crate::Error>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.finished {
            let blocks_ffi = &unsafe { slice::from_raw_parts(self.ffi, self.blocks_ffi_size) };
            let mut block_headers: Vec<_> = Vec::with_capacity(self.blocks_ffi_size);
            for block in blocks_ffi.iter() {
                let ffi = &unsafe { ptr::read(block) };
                block_headers.push(ffi.try_into());
            }
            Poll::Ready(block_headers)
        } else {
            cx.waker().wake_by_ref();
            self.get_mut().finished = true;
            Poll::Pending
        }
    }
}

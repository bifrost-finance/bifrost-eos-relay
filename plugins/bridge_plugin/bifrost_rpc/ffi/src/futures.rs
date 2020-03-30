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
	Action, ActionReceipt, BlockHeader, Checksum256, Extension,
	ProducerKey, ProducerSchedule, IncrementalMerkle, SignedBlockHeader
};
use super::ffi_types::*;
use std::{
	convert::TryInto,
	future::Future,
	task::{Context, Poll},
	pin::Pin,
};

pub(crate) struct ActionFuture<'a> {
	ffi: &'a ActionFFI,
	finished: bool,
}

impl<'a> Future for ActionFuture<'a> {
	type Output = Result<Action, crate::Error>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		if self.finished {
			let act: Result<Action, _> = self.ffi.try_into();
			Poll::Ready(act)
		} else {
			cx.waker().wake_by_ref();
			self.get_mut().finished = true;
			Poll::Pending
		}
	}
}

pub(crate) struct Checksum256Future<'a> {
	ffi: &'a Checksum256FFI,
	finished: bool,
}

impl<'a> Future for Checksum256Future<'a> {
	type Output = Result<Vec<Checksum256>, crate::Error>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		if self.finished {
			let ch_sums: Result<Vec<Checksum256>, _> = self.ffi.try_into();
			Poll::Ready(ch_sums)
		} else {
			cx.waker().wake_by_ref();
			self.get_mut().finished = true;
			Poll::Pending
		}
	}
}

pub(crate) struct ActionReceiptFuture<'a> {
	ffi: &'a ActionReceiptFFI,
	finished: bool,
}

impl<'a> Future for ActionReceiptFuture<'a> {
	type Output = Result<ActionReceipt, crate::Error>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		if self.finished {
			let receipt: Result<ActionReceipt, _> = self.ffi.try_into();
			Poll::Ready(receipt)
		} else {
			cx.waker().wake_by_ref();
			self.get_mut().finished = true;
			Poll::Pending
		}
	}
}

pub(crate) struct IncrementalMerkleFuture<'a> {
	ffi: &'a IncrementalMerkleFFI,
	finished: bool,
}

impl<'a> Future for IncrementalMerkleFuture<'a> {
	type Output = Result<IncrementalMerkle, crate::Error>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		if self.finished {
			let merkle: Result<IncrementalMerkle, _> = self.ffi.try_into();
			Poll::Ready(merkle)
		} else {
			cx.waker().wake_by_ref();
			self.get_mut().finished = true;
			Poll::Pending
		}
	}
}

pub(crate) struct ExtensionsFuture<'a> {
	ffi: &'a ExtensionsFFI,
	finished: bool,
}

impl<'a> Future for ExtensionsFuture<'a> {
	type Output = Result<Vec<Extension>, crate::Error>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		if self.finished {
			let exts: Result<Vec<Extension>, _> = self.ffi.try_into();
			Poll::Ready(exts)
		} else {
			cx.waker().wake_by_ref();
			self.get_mut().finished = true;
			Poll::Pending
		}
	}
}

pub(crate) struct ProducerKeyFuture<'a> {
	ffi: &'a ProducerKeyFFI,
	finished: bool,
}

impl<'a> Future for ProducerKeyFuture<'a> {
	type Output = Result<ProducerKey, crate::Error>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		if self.finished {
			let pk: Result<ProducerKey, _> = self.ffi.try_into();
			Poll::Ready(pk)
		} else {
			cx.waker().wake_by_ref();
			self.get_mut().finished = true;
			Poll::Pending
		}
	}
}

pub(crate) struct ProducerScheduleFuture<'a> {
	ffi: &'a ProducerScheduleFFI,
	finished: bool,
}

impl<'a> Future for ProducerScheduleFuture<'a> {
	type Output = Result<ProducerSchedule, crate::Error>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		if self.finished {
			let ps: Result<ProducerSchedule, _> = self.ffi.try_into();
			Poll::Ready(ps)
		} else {
			cx.waker().wake_by_ref();
			self.get_mut().finished = true;
			Poll::Pending
		}
	}
}

pub(crate) struct BlockHeaderFuture<'a> {
	ffi: &'a BlockHeaderFFI,
	finished: bool,
}

impl<'a> Future for BlockHeaderFuture<'a> {
	type Output = Result<BlockHeader, crate::Error>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		if self.finished {
			let header: Result<BlockHeader, _> = self.ffi.try_into();
			Poll::Ready(header)
		} else {
			cx.waker().wake_by_ref();
			self.get_mut().finished = true;
			Poll::Pending
		}
	}
}

pub(crate) struct SignedBlockHeaderFuture<'a> {
	ffi: &'a SignedBlockHeaderFFI,
	finished: bool,
}

impl<'a> Future for SignedBlockHeaderFuture<'a> {
	type Output = Result<SignedBlockHeader, crate::Error>;
	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		if self.finished {
			let signed_header: Result<SignedBlockHeader, _> = self.ffi.try_into();
			Poll::Ready(signed_header)
		} else {
			cx.waker().wake_by_ref();
			self.get_mut().finished = true;
			Poll::Pending
		}
	}
}

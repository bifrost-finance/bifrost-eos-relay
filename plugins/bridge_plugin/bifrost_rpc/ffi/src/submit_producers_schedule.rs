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

use codec::Encode;
use core::marker::PhantomData;
use eos_chain::ProducerAuthoritySchedule;
use subxt::{
	PairSigner, DefaultNodeRuntime as BifrostRuntime, Call, Client, Encoded,
	sudo::{Sudo, SudoEventsDecoder, SudoCall}, system::{AccountStoreExt, System, SystemEventsDecoder}
};
use sp_core::{sr25519::Pair, Pair as TraitPair};

#[subxt::module]
pub trait BridgeEos: System + Sudo {}

impl BridgeEos for BifrostRuntime {}

#[derive(Clone, Debug, PartialEq, Call, Encode)]
pub struct SaveProducerScheduleCall<T: BridgeEos + Sudo> {
	pub ps: ProducerAuthoritySchedule,
	pub _runtime: PhantomData<T>,
}

#[allow(dead_code)]
pub fn create_sudo_call<'a, T: Sudo>(call: &'a Encoded) -> SudoCall<T> {
	SudoCall {
		call,
		_runtime: PhantomData,
	}
}

#[allow(dead_code)]
pub async fn save_producer_schedule_call(
	url:                  impl AsRef<str>,
	signer:               impl AsRef<str>,
	schedule:             ProducerAuthoritySchedule
) -> Result<String, crate::Error> {
	let signer = Pair::from_string(signer.as_ref(), None).map_err(|_| crate::Error::WrongSudoSeed)?;
	let signer = PairSigner::<BifrostRuntime, Pair>::new(signer);

	let client: Client<BifrostRuntime> = subxt::ClientBuilder::new()
		.set_url(url.as_ref())
		.build()
		.await
		.map_err(|_| crate::Error::SubxtError("failed to create subxt client"))?;

	let args = SaveProducerScheduleCall {
		ps: schedule,
		_runtime: PhantomData,
	};

	let proposal = client.encode(args).map_err(|_| crate::Error::SubxtError("failed to encode args"))?;
	let call = create_sudo_call(&proposal);

	let extrinsic = client.watch(call, &signer).await.map_err(|_| crate::Error::SubxtError("failed to commit this transaction"))?;
	let block_hash = extrinsic.block;

	Ok(block_hash.to_string())
}

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
use eos_chain::{Action, ActionReceipt, Checksum256, IncrementalMerkle, SignedBlockHeader};
use subxt::{DefaultNodeRuntime as Runtime, Call, Client};
use sp_keyring::AccountKeyring;

#[derive(Encode)]
pub struct ChangeScheduleArgs {
	merkle: IncrementalMerkle,
	block_headers: Vec<SignedBlockHeader>,
	block_ids_list: Vec<Vec<Checksum256>>
}

#[derive(Encode)]
pub struct ProveActionArgs {
	action: Action,
	action_receipt: ActionReceipt,
	action_merkle_paths: Vec<Checksum256>,
	merkle: IncrementalMerkle,
	block_headers: Vec<SignedBlockHeader>,
	block_ids_list: Vec<Vec<Checksum256>>
}

pub async fn change_schedule_call(
	url: impl AsRef<str>,
	merkle: IncrementalMerkle,
	block_headers: Vec<SignedBlockHeader>,
	block_ids_list: Vec<Vec<Checksum256>>
) -> Result<String, subxt::Error> {
	let client: Client<Runtime> = subxt::ClientBuilder::new().set_url(url.as_ref()).build().await?;

	let signer = AccountKeyring::Alice.pair();

	let args = ChangeScheduleArgs {
		merkle,
		block_headers,
		block_ids_list,
	};

	let proposal = client.metadata().module_with_calls("BridgeEos").and_then(|module| module.call("change_schedule", args))?;
	let call = Call::new("Sudo", "sudo", proposal);
	let xt = client.xt(signer, None).await?;
	let hash = xt.submit(call).await?;

	Ok(hash.to_string())
}

pub async fn prove_action_call(
	url: impl AsRef<str>,
	action: Action,
	action_receipt: ActionReceipt,
	action_merkle_paths: Vec<Checksum256>,
	merkle: IncrementalMerkle,
	block_headers: Vec<SignedBlockHeader>,
	block_ids_list: Vec<Vec<Checksum256>>
) -> Result<String, subxt::Error> {
	let client: Client<Runtime> = subxt::ClientBuilder::new().set_url(url.as_ref()).build().await?;

	let signer = AccountKeyring::Alice.pair();

	let args = ProveActionArgs {
		action,
		action_receipt,
		action_merkle_paths,
		merkle,
		block_headers,
		block_ids_list,
	};

	let proposal = client.metadata().module_with_calls("BridgeEos").and_then(|module| module.call("prove_action", args))?;
	let call = Call::new("Sudo", "sudo", proposal);
	let xt = client.xt(signer, None).await?;
	let hash = xt.submit(call).await?;

	Ok(hash.to_string())
}

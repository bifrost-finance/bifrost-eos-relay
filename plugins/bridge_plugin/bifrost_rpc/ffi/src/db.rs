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

use codec::{Decode, Encode};
use eos_chain::{
	Action, ActionReceipt, Checksum256, Digest, IncrementalMerkle,
	ProducerAuthoritySchedule, SignedBlockHeader
};
use serde::{Deserialize, Serialize};

const DB_PATH: &str = "/home/sled/cross-chain";

#[derive(Clone, Debug, PartialEq, Decode, Encode, Deserialize, Serialize)]
pub struct ChangeScheduleArgs {
	legacy_schedule_hash: Checksum256,
	schedule:             ProducerAuthoritySchedule,
	merkle:               IncrementalMerkle,
	block_headers:        Vec<SignedBlockHeader>,
	block_ids_list:       Vec<Vec<Checksum256>>,
	block_id:             Option<String>, //this is block id from bifrost node if this transaction is submitted
}

#[derive(Clone, Debug, PartialEq, Decode, Encode, Deserialize, Serialize)]
pub struct ProveActionArgs {
	action:               Action,
	action_receipt:       ActionReceipt,
	action_merkle_paths:  Vec<Checksum256>,
	merkle:               IncrementalMerkle,
	block_headers:        Vec<SignedBlockHeader>,
	block_ids_list:       Vec<Vec<Checksum256>>,
	trx_id:               Checksum256,
	block_id:             Option<String>, //this is block id from bifrost node if this transaction is submitted
}

pub fn save_change_schedule_call(
	urls:                 impl IntoIterator<Item=String>,
	signer:               impl AsRef<str>,
	legacy_schedule_hash: Checksum256,
	schedule:             ProducerAuthoritySchedule,
	merkle:               IncrementalMerkle,
	block_headers:        Vec<SignedBlockHeader>,
	block_ids_list:       Vec<Vec<Checksum256>>
) -> Result<ProducerAuthoritySchedule, crate::Error> {
	let args = ChangeScheduleArgs {
		legacy_schedule_hash,
		schedule: schedule.clone(),
		merkle,
		block_headers,
		block_ids_list,
		block_id: None,
	};
	println!("ChangeScheduleArgs: {:?}", args);

	let tree = sled::open(DB_PATH).expect("failed to open sled db");
	if let Ok(Some((key, val))) = tree.last() {
		let last_index: u64 = String::from_utf8_lossy(key.as_ref())
			.parse()
			.map_err(|_| crate::Error::SubxtError("failed to get client builder"))?;

		let args_str = serde_json::to_vec(&args)
			.map_err(|_| crate::Error::SubxtError("failed to serialize extrinsic"))?;
		let latest_index = last_index + 1;
		tree.insert(latest_index.to_string().as_str(), args_str);
	} else {
		let args_str = serde_json::to_vec(&args)
			.map_err(|_| crate::Error::SubxtError("failed to serialize extrinsic"))?;
		let latest_index = 0;
		tree.insert(latest_index.to_string().as_str(), args_str);
	}

	Ok(schedule)
}

pub fn save_prove_action_call(
	urls:                impl IntoIterator<Item=String>,
	signer:              impl AsRef<str>,
	action:              Action,
	action_receipt:      ActionReceipt,
	action_merkle_paths: Vec<Checksum256>,
	merkle:              IncrementalMerkle,
	block_headers:       Vec<SignedBlockHeader>,
	block_ids_list:      Vec<Vec<Checksum256>>,
	trx_id:              Checksum256
) -> Result<(), crate::Error> {
	let args = ProveActionArgs {
		action: action.clone(),
		action_receipt,
		action_merkle_paths,
		merkle,
		block_headers,
		block_ids_list,
		trx_id,
		block_id: None,
	};
	println!("ProveActionArgs: {:?}", args);

	let tree = sled::open(DB_PATH).expect("failed to open sled db");
	if let Ok(Some((key, _))) = tree.last() {
		let last_index: u64 = String::from_utf8_lossy(key.as_ref())
			.parse()
			.map_err(|_| crate::Error::SubxtError("failed to get client builder"))?;

		let args_str = serde_json::to_vec(&args)
			.map_err(|_| crate::Error::SubxtError("failed to serialize extrinsic"))?;
		let latest_index = last_index + 1;
		tree.insert(latest_index.to_string().as_str(), args_str);
	} else {
		let args_str = serde_json::to_vec(&args)
			.map_err(|_| crate::Error::SubxtError("failed to serialize extrinsic"))?;
		let latest_index = 0;
		tree.insert(latest_index.to_string().as_str(), args_str);
	}

	Ok(())
}

// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use super::hash::H256;
use serde::{Deserialize, Deserializer, Error};
use serde::de::Visitor;
use ethstore;

/// Type of derivation
pub enum DerivationType {
	/// Soft - allow proof of parent
	Soft,
	/// Hard - does not allow proof of parent
	Hard,
}

/// Derivation request by hash
#[derive(Deserialize)]
pub struct DeriveHash {
	hash: H256,
	#[serde(rename="type")]
	d_type: DerivationType,
}

/// Node propertoes in hierarchical derivation request
#[derive(Deserialize)]
pub struct DeriveHierarchicalItem {
	index: u64,
	#[serde(rename="type")]
	d_type: DerivationType,
}

/// Hierarchical (index sequence) request
pub type DeriveHierarchical = Vec<DeriveHierarchicalItem>;

/// Generic derivate request
pub enum Derive {
	/// Hierarchical (index sequence) request
	Hierarchical(DeriveHierarchical),
	/// Hash request
	Hash(DeriveHash),
}

impl From<DeriveHierarchical> for Derive {
	fn from(d: DeriveHierarchical) -> Self {
		Derive::Hierarchical(d)
	}
}

impl From<DeriveHash> for Derive {
	fn from(d: DeriveHash) -> Self {
		Derive::Hash(d)
	}
}

/// Error converting request data
#[derive(Debug)]
pub enum ConvertError {
	IndexOverlfow(u64),
}

impl Derive {
	/// Convert to account provider struct dealing with possible overflows
	pub fn to_derivation(self) -> Result<ethstore::Derivation, ConvertError> {
		Ok(match self {
			Derive::Hierarchical(drv) => {
				ethstore::Derivation::Hierarchical({
					let mut members = Vec::<ethstore::IndexDerivation>::new();
					for h in drv {
						if h.index > ::std::u32::MAX as u64 { return Err(ConvertError::IndexOverlfow(h.index)); }
						members.push(match h.d_type {
							DerivationType::Soft => ethstore::IndexDerivation { soft: true, index: h.index as u32 },
							DerivationType::Hard => ethstore::IndexDerivation { soft: false, index: h.index as u32 },
						});
					}
					members
			   })
			},
			Derive::Hash(drv) => {
				match drv.d_type {
					DerivationType::Soft => ethstore::Derivation::SoftHash(drv.hash.into()),
					DerivationType::Hard => ethstore::Derivation::HardHash(drv.hash.into()),
				}
			},
		})
	}
}

impl Deserialize for DerivationType {
	fn deserialize<D>(deserializer: &mut D) -> Result<DerivationType, D::Error>
	where D: Deserializer {
		deserializer.deserialize(DerivationTypeVisitor)
	}
}

struct DerivationTypeVisitor;

impl Visitor for DerivationTypeVisitor {
	type Value = DerivationType;

	fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: Error {
		match value {
			"soft" => Ok(DerivationType::Soft),
			"hard" => Ok(DerivationType::Hard),
			_ => Err(Error::custom("invalid derivation type")),
		}
	}

	fn visit_string<E>(&mut self, value: String) -> Result<Self::Value, E> where E: Error {
		self.visit_str(value.as_ref())
	}
}
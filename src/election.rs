use serde::{Deserialize, Serialize};
use tfhe::prelude::*;
use tfhe::FheUint8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Candidate {
    Alice = 0,
    Bob = 1,
}

pub const NUM_CANDIDATES: usize = 2;

#[derive(Clone, Serialize, Deserialize)]
pub struct Ballot {
    pub encrypted_choice: FheUint8,
}

impl Ballot {
    pub fn try_new(choice: Candidate, client_key: &tfhe::ClientKey) -> tfhe::Result<Self> {
        Ok(Ballot {
            encrypted_choice: FheUint8::try_encrypt(choice as u8, client_key)?,
        })
    }
}

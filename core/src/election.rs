use serde::{Deserialize, Serialize};
use tfhe::prelude::*;
use tfhe::FheUint8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Candidate {
    Alice = 0,
    Bob = 1,
}

pub const NUM_CANDIDATES: usize = 2;

pub const ALL_CANDIDATES: [Candidate; NUM_CANDIDATES] = [Candidate::Alice, Candidate::Bob];

#[derive(Debug)]
pub struct ParseCandidateError;

impl std::str::FromStr for Candidate {
    type Err = ParseCandidateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "alice" => Ok(Candidate::Alice),
            "bob" => Ok(Candidate::Bob),
            _ => Err(ParseCandidateError),
        }
    }
}

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

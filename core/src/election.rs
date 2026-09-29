use serde::{Deserialize, Serialize};
use tfhe::FheUint8;
use tfhe::prelude::*;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_candidate_names_case_insensitively() {
        assert_eq!("alice".parse::<Candidate>().unwrap(), Candidate::Alice);
        assert_eq!("Bob".parse::<Candidate>().unwrap(), Candidate::Bob);
        assert_eq!("ALICE".parse::<Candidate>().unwrap(), Candidate::Alice);
    }

    #[test]
    fn rejects_unknown_candidates() {
        assert!("carol".parse::<Candidate>().is_err());
        assert!("".parse::<Candidate>().is_err());
    }

    /// The tally uses `candidate as usize` as an index into the counts, so
    /// ALL_CANDIDATES must list every candidate at the position of its ID.
    #[test]
    fn candidate_ids_match_their_position() {
        for (index, candidate) in ALL_CANDIDATES.iter().enumerate() {
            assert_eq!(*candidate as usize, index);
        }
    }
}

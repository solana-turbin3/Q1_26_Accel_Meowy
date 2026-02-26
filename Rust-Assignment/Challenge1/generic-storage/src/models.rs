use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub age: u32,
}

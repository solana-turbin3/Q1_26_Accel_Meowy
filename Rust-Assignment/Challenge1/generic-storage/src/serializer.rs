use borsh::{BorshDeserialize, BorshSerialize};
use serde::{de::DeserializeOwned, Serialize};

pub trait Serializer {
    fn to_bytes<T>(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>>
    where
        T: BorshSerialize + Serialize;

    fn from_bytes<T>(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>>
    where
        T: BorshDeserialize + DeserializeOwned;
}

pub struct Borsh;

impl Serializer for Borsh {
    fn to_bytes<T>(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>>
    where
        T: BorshSerialize + Serialize,
    {
        Ok(borsh::to_vec(value)?)
    }

    fn from_bytes<T>(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>>
    where
        T: BorshDeserialize + DeserializeOwned,
    {
        Ok(borsh::from_slice(bytes)?)
    }
}

pub struct Bincode;

impl Serializer for Bincode {
    fn to_bytes<T>(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>>
    where
        T: BorshSerialize + Serialize,
    {
        Ok(bincode::serialize(value)?)
    }

    fn from_bytes<T>(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>>
    where
        T: BorshDeserialize + DeserializeOwned,
    {
        Ok(bincode::deserialize(bytes)?)
    }
}

pub struct Json;

impl Serializer for Json {
    fn to_bytes<T>(&self, value: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>>
    where
        T: BorshSerialize + Serialize,
    {
        Ok(serde_json::to_vec(value)?)
    }

    fn from_bytes<T>(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>>
    where
        T: BorshDeserialize + DeserializeOwned,
    {
        Ok(serde_json::from_slice(bytes)?)
    }
}

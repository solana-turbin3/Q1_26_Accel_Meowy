use std::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{de::DeserializeOwned, Serialize};

use crate::serializer::Serializer;

pub struct Storage<T, S>
where
    T: BorshSerialize + BorshDeserialize + Serialize + DeserializeOwned,
    S: Serializer,
{
    serializer: S,
    data: Option<Vec<u8>>,
    _marker: PhantomData<T>,
}

impl<T, S> Storage<T, S>
where
    T: BorshSerialize + BorshDeserialize + Serialize + DeserializeOwned,
    S: Serializer,
{
    pub fn new(serializer: S) -> Self {
        Storage {
            serializer,
            data: None,
            _marker: PhantomData,
        }
    }

    pub fn save(&mut self, value: &T) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = self.serializer.to_bytes(value)?;
        self.data = Some(bytes);
        Ok(())
    }

    pub fn load(&self) -> Result<T, Box<dyn std::error::Error>> {
        match &self.data {
            Some(bytes) => self.serializer.from_bytes(bytes),
            None => Err("No data stored".into()),
        }
    }

    pub fn has_data(&self) -> bool {
        self.data.is_some()
    }
}

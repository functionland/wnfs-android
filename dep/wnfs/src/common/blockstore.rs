//! Block store traits.

use std::{borrow::Cow, io::Cursor};

use anyhow::Result;
use async_trait::async_trait;
use hashbrown::HashMap;
use libipld::{
    cbor::DagCborCodec,
    cid::Version,
    codec::{Decode, Encode},
    multihash::{Code, MultihashDigest},
    serde as ipld_serde, Cid, Ipld, IpldCodec,
};
use rand_core::RngCore;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    private::{Key, NONCE_SIZE},
    utils, AsyncSerialize,
};

use super::FsError;

//--------------------------------------------------------------------------------------------------
// Type Definitions
//--------------------------------------------------------------------------------------------------

/// For types that implement block store operations like adding, getting content from the store.
#[async_trait(?Send)]
pub trait BlockStore {
    async fn get_block<'a>(&'a self, cid: &Cid) -> Result<Cow<'a, Vec<u8>>>;
    async fn put_block(&mut self, bytes: Vec<u8>, codec: IpldCodec) -> Result<Cid>;

    async fn put_serializable<V: Serialize>(&mut self, value: &V) -> Result<Cid> {
        let ipld = ipld_serde::to_ipld(value)?;

        let mut bytes = Vec::new();
        ipld.encode(DagCborCodec, &mut bytes)?;

        self.put_block(bytes, IpldCodec::DagCbor).await
    }

    async fn put_private_serializable<V, R>(
        &mut self,
        value: &V,
        key: &Key,
        rng: &mut R,
    ) -> Result<Cid>
    where
        V: Serialize,
        R: RngCore,
    {
        let ipld = ipld_serde::to_ipld(value)?;
        let mut bytes = Vec::new();
        ipld.encode(DagCborCodec, &mut bytes)?;
        let enc_bytes = key.encrypt(&utils::get_random_bytes::<NONCE_SIZE>(rng), &bytes)?;
        self.put_block(enc_bytes, IpldCodec::DagCbor).await
    }

    async fn put_async_serializable<V: AsyncSerialize>(&mut self, value: &V) -> Result<Cid> {
        let ipld = value.async_serialize_ipld(self).await?;

        let mut bytes = Vec::new();
        ipld.encode(DagCborCodec, &mut bytes)?;

        self.put_block(bytes, IpldCodec::DagCbor).await
    }

    async fn get_deserializable<'a, V: DeserializeOwned>(&'a self, cid: &Cid) -> Result<V> {
        let bytes = self.get_block(cid).await?;
        let ipld = Ipld::decode(DagCborCodec, &mut Cursor::new(bytes.as_ref()))?;
        Ok(ipld_serde::from_ipld::<V>(ipld)?)
    }

    async fn get_private_deserializable<'a, V: DeserializeOwned>(
        &'a self,
        cid: &Cid,
        key: &Key,
    ) -> Result<V> {
        let enc_bytes = self.get_block(cid).await?;
        let bytes = key.decrypt(enc_bytes.as_ref())?;
        let ipld = Ipld::decode(DagCborCodec, &mut Cursor::new(bytes))?;
        Ok(ipld_serde::from_ipld::<V>(ipld)?)
    }
}

/// An in-memory block store to simulate IPFS.
///
/// IPFS is basically a glorified HashMap.
#[derive(Debug, Default)]
pub struct MemoryBlockStore(HashMap<String, Vec<u8>>);

//--------------------------------------------------------------------------------------------------
// Implementations
//--------------------------------------------------------------------------------------------------

impl MemoryBlockStore {
    /// Creates a new in-memory block store.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait(?Send)]
impl BlockStore for MemoryBlockStore {
    /// Stores an array of bytes in the block store.
    async fn put_block(&mut self, bytes: Vec<u8>, codec: IpldCodec) -> Result<Cid> {
        let hash = Code::Sha2_256.digest(&bytes);
        let cid = Cid::new(Version::V1, codec.into(), hash)?;

        self.0.insert(cid.to_string(), bytes);

        Ok(cid)
    }

    /// Retrieves an array of bytes from the block store with given CID.
    async fn get_block<'a>(&'a self, cid: &Cid) -> Result<Cow<'a, Vec<u8>>> {
        let bytes = self
            .0
            .get(&cid.to_string())
            .ok_or(FsError::CIDNotFoundInBlockstore)?;

        Ok(Cow::Borrowed(bytes))
    }
}

//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use libipld::{cbor::DagCborCodec, codec::Encode};

    use super::*;

    #[async_std::test]
    async fn inserted_items_can_be_fetched() {
        let store = &mut MemoryBlockStore::new();

        let first_bytes = {
            let mut tmp = vec![];
            vec![1, 2, 3, 4, 5]
                .to_vec()
                .encode(DagCborCodec, &mut tmp)
                .unwrap();
            tmp
        };

        let second_bytes = {
            let mut tmp = vec![];
            b"hello world"
                .to_vec()
                .encode(DagCborCodec, &mut tmp)
                .unwrap();
            tmp
        };

        let first_cid = &store
            .put_block(first_bytes, IpldCodec::DagCbor)
            .await
            .unwrap();

        let second_cid = &store
            .put_block(second_bytes, IpldCodec::DagCbor)
            .await
            .unwrap();

        let first_loaded: Vec<u8> = store.get_deserializable(first_cid).await.unwrap();
        let second_loaded: Vec<u8> = store.get_deserializable(second_cid).await.unwrap();

        assert_eq!(first_loaded, vec![1, 2, 3, 4, 5]);
        assert_eq!(second_loaded, b"hello world".to_vec());
    }
}

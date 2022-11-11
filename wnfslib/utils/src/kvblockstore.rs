use kv::*;
use std::{borrow::Cow};
use libipld::{
    cid::Version,
    Cid, IpldCodec,
};
use anyhow::Result;
use async_trait::async_trait;
use multihash::{Code, MultihashDigest};

use wnfs::FsError;
use wnfs::BlockStore;

pub struct KVBlockStore{
    pub store: Store
} 

//--------------------------------------------------------------------------------------------------
// Implementations
//--------------------------------------------------------------------------------------------------

impl KVBlockStore {
    /// Creates a new kv block store.
    pub fn new(db_path: String) -> Self {
        // Configure the database
        // Open the key/value store
        Self{
            store: Store::new(Config::new(db_path)).unwrap()
        }
    }

}


#[async_trait(?Send)]
impl BlockStore for KVBlockStore {
    /// Retrieves an array of bytes from the block store with given CID.
    async fn get_block<'a>(&'a self, cid: &Cid) -> Result<Cow<'a, Vec<u8>>> {
        // A Bucket provides typed access to a section of the key/value store
        let bucket = self.store.bucket::<Raw, Raw>(Some("default"))?;

        let bytes = bucket
            .get(&Raw::from(cid.to_bytes()))
            .map_err(|_| FsError::CIDNotFoundInBlockstore)?.unwrap().to_vec();
        Ok(Cow::Owned(bytes))
    }

    /// Stores an array of bytes in the block store.
    async fn put_block(&mut self, bytes: Vec<u8>, codec: IpldCodec) -> Result<Cid> {

        let hash = Code::Sha2_256.digest(&bytes);
        let cid = Cid::new(Version::V1, codec.into(), hash)?;

        let key = Raw::from(cid.to_bytes());
        let value = Raw::from(bytes);
        // A Bucket provides typed access to a section of the key/value store
        let bucket = self.store.bucket::<Raw, Raw>(Some("default"))?;

        bucket.set(&key, &value)?;
        Ok(cid)
    }
}



//--------------------------------------------------------------------------------------------------
// Functions
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod blockstore_tests {
    use libipld::{cbor::DagCborCodec, codec::Encode, IpldCodec};

    use wnfs::*;

    use crate::kvblockstore::KVBlockStore;

    #[async_std::test]
    async fn inserted_items_can_be_fetched() {
        let store = &mut KVBlockStore::new(String::from("./tmp/test2"));

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

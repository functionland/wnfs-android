use std::{borrow::Cow};
use libipld::{
    Cid, IpldCodec,
};
use anyhow::Result;
use async_trait::async_trait;

use wnfs::FsError;
use wnfs::BlockStore;

pub trait FFIStore<'a> {
    fn get_block<'b>(&'b self, cid: Vec<u8>) -> Result<Vec<u8>>;
    fn put_block<'b>(&'b self, bytes: Vec<u8>, codec: i64) -> Result<Vec<u8>>;
}

pub struct FFIFriendlyBlockStore<'a>{
    pub ffi_store: Box<dyn FFIStore<'a> + 'a>
} 

//--------------------------------------------------------------------------------------------------
// Implementations
//--------------------------------------------------------------------------------------------------

impl<'a> FFIFriendlyBlockStore<'a> {
    /// Creates a new kv block store.
    pub fn new(ffi_store: Box<dyn FFIStore<'a> + 'a>) -> Self
    {
        Self{
            ffi_store
        }
    }
}


#[async_trait(?Send)]
impl<'b> BlockStore for FFIFriendlyBlockStore<'b> {
    /// Retrieves an array of bytes from the block store with given CID.
    async fn get_block<'a>(&'a self, cid: &Cid) -> Result<Cow<'a, Vec<u8>>> {
        let bytes = self.ffi_store.get_block(cid.to_bytes())
            .map_err(|_| FsError::CIDNotFoundInBlockstore)?;
        Ok(Cow::Owned(bytes))
    }

    /// Stores an array of bytes in the block store.
    async fn put_block(&mut self, bytes: Vec<u8>, codec: IpldCodec) -> Result<Cid> {
        let codec_u64: u64 = codec.into();
        //let codec_vu8: Vec<u8> = codec_u64.to_be_bytes().to_vec();
        let codec_i64: i64 = i64::try_from(codec_u64).unwrap();
        let cid_bytes = self.ffi_store.put_block(bytes.to_owned(), codec_i64)?;
        let cid = Cid::try_from(cid_bytes).unwrap();
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

    use crate::{kvstore::KVBlockStore, blockstore::FFIFriendlyBlockStore};

    #[async_std::test]
    async fn inserted_items_can_be_fetched() {
        let store = KVBlockStore::new(String::from("./tmp/test1"), IpldCodec::DagCbor);
        let blockstore = &mut FFIFriendlyBlockStore::new(Box::new(store));
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

        let first_cid = &blockstore
            .put_block(first_bytes, IpldCodec::DagCbor)
            .await
            .unwrap();

        let second_cid = &blockstore
            .put_block(second_bytes, IpldCodec::DagCbor)
            .await
            .unwrap();

        let first_loaded: Vec<u8> = blockstore.get_deserializable(first_cid).await.unwrap();
        let second_loaded: Vec<u8> = blockstore.get_deserializable(second_cid).await.unwrap();

        assert_eq!(first_loaded, vec![1, 2, 3, 4, 5]);
        assert_eq!(second_loaded, b"hello world".to_vec());
    }
}

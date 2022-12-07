use kv::*;

use std::convert::TryInto;
use libipld::{
    cid::Version,
    Cid, IpldCodec,
    multihash::MultihashGeneric,
    multihash::MultihashDigest,
};
use anyhow::Result;

use wnfs::FsError;

use crate::blockstore::FFIStore;

pub struct KVBlockStore{
    pub store: Store,
    pub codec: IpldCodec
} 

fn vec_to_array<T>(v: Vec<T>) -> [T; 8] where T: Copy {
    let slice = v.as_slice();
    let array: [T; 8] = match slice.try_into() {
        Ok(ba) => ba,
        Err(_) => panic!("Expected a Vec of length {} but it was {}", 8, v.len()),
    };
    array
}

//--------------------------------------------------------------------------------------------------
// Implementations
//--------------------------------------------------------------------------------------------------

impl KVBlockStore {
    /// Creates a new kv block store.
    pub fn new(db_path: String, codec: IpldCodec) -> Self {
        // Configure the database
        // Open the key/value store
        Self{
            store: Store::new(Config::new(db_path)).unwrap(),
            codec
        }
    }

}


impl<'a> FFIStore<'a> for KVBlockStore {
    /// Retrieves an array of bytes from the block store with given CID.
    fn get_block(&self, cid: Vec<u8>) -> Result<Vec<u8>>{
        // A Bucket provides typed access to a section of the key/value store
        let bucket = self.store.bucket::<Raw, Raw>(Some("default"))?;

        let bytes = bucket
            .get(&Raw::from(cid))
            .map_err(|_| FsError::CIDNotFoundInBlockstore)?.unwrap().to_vec();
        Ok(bytes)
    }

    /// Stores an array of bytes in the block store.
    fn put_block(&self, bytes: Vec<u8>, codec: i64) -> Result<Vec<u8>>{
        //let codec_u8_array:[u8;8] = vec_to_array(codec);
        //let codec_u64 = u64::from_be_bytes(codec_u8_array);
        let codec_u64: u64 = u64::try_from(codec).unwrap();
        let hash: MultihashGeneric<64> = multihash::Code::Sha2_256.digest(&bytes);
        let codec = IpldCodec::try_from(codec_u64).unwrap();
        let cid = Cid::new(Version::V1, codec.into(), hash)?;

        let cid_bytes = cid.to_bytes();
        let key = Raw::from(cid_bytes.to_owned());
        let value = Raw::from(bytes);
        
        // A Bucket provides typed access to a section of the key/value store
        let bucket = self.store.bucket::<Raw, Raw>(Some("default"))?;

        bucket.set(&key, &value)?;
        Ok(cid_bytes)
    }
}

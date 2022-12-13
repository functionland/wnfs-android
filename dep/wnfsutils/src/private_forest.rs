//! This example shows how to add a directory to a private forest (also HAMT) which encrypts it.
//! It also shows how to retrieve encrypted nodes from the forest using `PrivateRef`s.

use chrono::Utc;
use libipld::Cid;
use rand::{thread_rng, rngs::ThreadRng};
use std::{
    rc::Rc, 
    fs::{File, OpenOptions}, 
    io::{Read, Write}
};
use wnfs::{
    dagcbor, Hasher, utils,
    private::{PrivateForest, PrivateRef, PrivateNode, Key},
    BlockStore, Namefilter, PrivateDirectory, PrivateOpResult, Metadata,
};
use anyhow::Result;
use log::{trace, Level};
use sha3::Sha3_256;


use crate::blockstore::FFIFriendlyBlockStore;

pub struct PrivateDirectoryHelper<'a> {
    pub store: FFIFriendlyBlockStore<'a>,
    rng: ThreadRng
}

// Single root (private ref) implementation of the wnfs private directory using KVBlockStore.
// TODO: we assumed all the write, mkdirs use same roots here. this could be done using prepend
// a root path to all path segments.
impl<'a> PrivateDirectoryHelper<'a> {
    pub fn new(block_store: FFIFriendlyBlockStore<'a>) -> Self
    where
    
     {
        Self { 
            store: block_store,
            rng: thread_rng()
        }
    }

    pub async fn create_private_forest(&mut self) -> Result<Cid> {
        // Create the private forest (also HAMT), a map-like structure where files and directories are stored.
        let forest = Rc::new(PrivateForest::new());
        
        // Serialize the private forest to DAG CBOR.
        let cbor_bytes = dagcbor::async_encode(&forest, &mut self.store).await.unwrap();

        // Persist encoded private forest to the block store.
        self.store.put_serializable(&cbor_bytes).await
    }

    pub async fn load_forest(&mut self, forest_cid: Cid) -> Result<Rc<PrivateForest>> {
        // Fetch CBOR bytes of private forest from the blockstore.
        let cbor_bytes = self.store
            .get_deserializable::<Vec<u8>>(&forest_cid)
            .await
            .unwrap();

        // Decode private forest CBOR bytes.
        Ok(Rc::new(dagcbor::decode::<PrivateForest>(cbor_bytes.as_ref()).unwrap()))
    }

    pub async fn update_forest(&mut self, hamt: Rc<PrivateForest>) -> Result<Cid> {
        // Serialize the private forest to DAG CBOR.
        let cbor_bytes = dagcbor::async_encode(&hamt, &mut self.store).await.unwrap();

        // Persist encoded private forest to the block store.
        self.store.put_serializable(&cbor_bytes).await
    }

    pub async fn get_root_dir(&mut self, forest: Rc<PrivateForest>, private_ref: PrivateRef) -> Result<Rc<PrivateDirectory>> {
        //trace!("\r\n wnfs13 revision_key = {:?}", private_ref.revision_key.0.as_bytes());
        //trace!("\r\n wnfs13 saturated_name_hash = {:?}", private_ref.saturated_name_hash);
        //trace!("\r\n wnfs13 content_key = {:?}", private_ref.content_key.0.as_bytes());
        // Fetch and decrypt root directory from the private forest using provided private ref.
        forest
        .get(&private_ref, PrivateForest::resolve_lowest, &mut self.store)
        .await
        .unwrap().unwrap().as_dir()
    }

    pub async fn get_private_ref(&mut self, wnfs_key: Vec<u8>) -> PrivateRef {
        let ratchet_seed: [u8; 32] = Sha3_256::hash(&wnfs_key);
        let inumber: [u8; 32] = Sha3_256::hash(&ratchet_seed);
        let private_ref = PrivateRef::with_seed(Namefilter::default(), ratchet_seed, inumber);
        trace!("\r\n wnfs13 get_private_ref.content_key {:?}", private_ref.content_key.0.as_bytes());
        trace!("\r\n wnfs13 get_private_ref.saturated_name_hash {:?}", private_ref.saturated_name_hash);
        trace!("\r\n wnfs13 get_private_ref.revision_key {:?}", private_ref.revision_key.0.as_bytes());

        private_ref
        
    }

    pub async fn init(&mut self, forest: Rc<PrivateForest>, wnfs_key: Vec<u8>) -> (Cid, PrivateRef) {
        let ratchet_seed: [u8; 32];
        let inumber: [u8; 32];
        if wnfs_key.is_empty() {
            let wnfs_random_key = Key::new(utils::get_random_bytes::<32>(&mut self.rng));
            ratchet_seed = Sha3_256::hash(&wnfs_random_key.as_bytes());
            inumber = utils::get_random_bytes::<32>(&mut self.rng); // Needs to be random
        }else {
            ratchet_seed = Sha3_256::hash(&wnfs_key);
            inumber = Sha3_256::hash(&ratchet_seed);
        }

        // Create a new directory.
        /*let dir = Rc::new(PrivateDirectory::with_seed(
                Namefilter::default(),
                Utc::now(),
                ratchet_seed,
                inumber
            ));*/
        let dir = PrivateNode::from(PrivateDirectory::with_seed(
            Namefilter::default(),
            Utc::now(),
            ratchet_seed,
            inumber,
        ));
        let header = dir.get_header();

        trace!("\r\n wnfs13 header revision_key = {:?}", header.get_private_ref().revision_key.0.as_bytes());
        trace!("\r\n wnfs13 header saturated_name_hash = {:?}", header.get_private_ref().saturated_name_hash);
        trace!("\r\n wnfs13 header content_key = {:?}", header.get_private_ref().content_key.0.as_bytes());

        let forest = forest
            .put(
                header.get_saturated_name(),
                &header.get_private_ref(),
                &dir,
                &mut self.store,
                &mut self.rng,
            )
            .await
            .unwrap();

            trace!("\r\n wnfs13 init1 revision_key = {:?}", dir.as_dir().unwrap().header.get_private_ref().revision_key.0.as_bytes());
            trace!("\r\n wnfs13 init1 saturated_name_hash = {:?}", dir.as_dir().unwrap().header.get_private_ref().saturated_name_hash);
            trace!("\r\n wnfs13 init1 content_key = {:?}", dir.as_dir().unwrap().header.get_private_ref().content_key.0.as_bytes());

        let PrivateOpResult { root_dir, forest, .. } = dir
            .as_dir()
            .unwrap()
            .mkdir(&["root".into()], true, Utc::now(), forest, &mut self.store,&mut self.rng)
            .await
            .unwrap();
        let init_private_ref = root_dir.header.get_private_ref();

        trace!("\r\n wnfs13 init2 revision_key = {:?}", init_private_ref.revision_key.0.as_bytes());
            trace!("\r\n wnfs13 init2 saturated_name_hash = {:?}", init_private_ref.saturated_name_hash);
            trace!("\r\n wnfs13 init2 content_key = {:?}", init_private_ref.content_key.0.as_bytes());
        
        (self.update_forest(forest).await.unwrap(), init_private_ref)
    }

    fn get_file_as_byte_vec(&mut self, filename: &String) -> Vec<u8> {
        let mut f = File::open(&filename).expect("no file found");
        let metadata = std::fs::metadata(&filename).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");
    
        buffer
    }

    pub async fn write_file_from_path(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], filename: &String) -> (Cid, PrivateRef) {
        let content: Vec<u8> = self.get_file_as_byte_vec(filename);
        self.write_file(forest, root_dir, path_segments, content).await
    }

    fn write_byte_vec_to_file(&mut self, filename: &String, file_content: Vec<u8>) {
        trace!("wnfs11 **********************write_byte_vec_to_file started**************filename={:?}", filename);
        trace!("wnfs11 **********************write_byte_vec_to_file started**************file_content={:?}", file_content);
        let mut file = File::create(filename).unwrap_or_else(|_err: std::io::Error| {
            trace!("**********************put_block first unwrap error**************");
            panic!("HERE1: {:?}", _err)
        });
        trace!("wnfs11 **********************write_byte_vec_to_file write created**************");
        file
        .write_all(&file_content)
        .expect("Unable to write data");
        
    }

    pub async fn write_file(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], content: Vec<u8>) -> (Cid, PrivateRef) {
        let PrivateOpResult { forest, root_dir, .. } = root_dir
            .write(
                path_segments,
                true,
                Utc::now(),
                content,
                forest,
                &mut self.store,
                &mut self.rng,
            )
            .await
            .unwrap();
        (self.update_forest(forest).await.unwrap(), root_dir.header.get_private_ref())
    }

    pub async fn read_file_to_path(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], filename: &String) -> String {
        let file_content = self.read_file(forest, root_dir, path_segments).await.unwrap();
        self.write_byte_vec_to_file(filename, file_content);
        filename.to_string()
    }

    pub async fn read_file(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Option<Vec<u8>> {
        let result = root_dir
            .read(path_segments, true, forest, &mut self.store)
            .await;
        match result {
            Ok(res) => Some(res.result),
            Err(_) => None
        }
    }


    pub async fn mkdir(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> (Cid, PrivateRef) {
        let PrivateOpResult { forest, root_dir, .. } = root_dir
            .mkdir(path_segments, true, Utc::now(), forest, &mut self.store,&mut self.rng)
            .await
            .unwrap();

        (self.update_forest(forest).await.unwrap(), root_dir.header.get_private_ref())
    }


    pub async fn rm(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> (Cid, PrivateRef) {
        let PrivateOpResult { forest, root_dir, .. } = root_dir
            .rm(path_segments, true, forest, &mut self.store,&mut self.rng)
            .await
            .unwrap();

        (self.update_forest(forest).await.unwrap(), root_dir.header.get_private_ref())
    }

    pub async fn ls_files(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Vec<(String, Metadata)> {
        let PrivateOpResult { result, .. } = root_dir
            .ls(path_segments, true, forest, &mut self.store)
            .await
            .unwrap();
        result
    }

}

// Implement synced version of the library for using in android jni.
impl<'a> PrivateDirectoryHelper<'a> {
    pub fn synced_create_private_forest(&mut self) -> Result<Cid>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.create_private_forest());
    }

    pub fn synced_load_forest(&mut self, forest_cid: Cid) -> Result<Rc<PrivateForest>>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.load_forest(forest_cid));
    }

    pub fn synced_get_private_ref(&mut self, wnfs_key: Vec<u8>) -> PrivateRef
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.get_private_ref(wnfs_key));
    }


    pub fn synced_get_root_dir(&mut self, forest: Rc<PrivateForest>, private_ref: PrivateRef) -> Result<Rc<PrivateDirectory>, anyhow::Error>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.get_root_dir(forest, private_ref));
    }

    pub fn synced_init(&mut self, forest: Rc<PrivateForest>, wnfs_key: Vec<u8>) -> (Cid, PrivateRef)
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.init(forest, wnfs_key));
    }

    pub fn synced_write_file_from_path(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], filename: &String) -> (Cid, PrivateRef)
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.write_file_from_path(forest, root_dir, path_segments, filename));
    }

    pub fn synced_write_file(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], content: Vec<u8>) -> (Cid, PrivateRef)
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.write_file(forest, root_dir, path_segments, content));
    }

    pub fn synced_read_file_to_path(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], filename: &String) -> String
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.read_file_to_path(forest, root_dir, path_segments, filename));
    }

    pub fn synced_read_file(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Option<Vec<u8>>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.read_file(forest, root_dir, path_segments));
    }

    pub fn synced_mkdir(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> (Cid, PrivateRef)
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.mkdir(forest, root_dir, path_segments));
    }

    pub fn synced_rm(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> (Cid, PrivateRef)
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.rm(forest, root_dir, path_segments));
    }

    pub fn synced_ls_files(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Vec<(String, Metadata)>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.ls_files(forest, root_dir, path_segments));
    }

    pub fn parse_path(path: String) -> Vec<String> {
        path.trim().trim_matches('/').split("/").
        map(|s| s.to_string()).
        collect()
    }

}

#[cfg(test)]
mod private_tests {
    use libipld::IpldCodec;

    use crate::{private_forest::PrivateDirectoryHelper, kvstore::KVBlockStore, blockstore::FFIFriendlyBlockStore};


    #[async_std::test]
    async fn test_parse_path() {
        let path = "root/test.txt".to_string();
        let out = PrivateDirectoryHelper::parse_path(path);
        assert_eq!(out[0], "root".to_string());
        assert_eq!(out[1], "test.txt".to_string());
    }

    #[async_std::test]
    async fn iboverall() {
        let empty_key: Vec<u8> = vec![0; 32];
        let store = KVBlockStore::new(String::from("./tmp/test2"), IpldCodec::DagCbor);
        let blockstore = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(blockstore);
        let forest_cid = helper.create_private_forest().await.unwrap();
        println!("cid: {:?}", forest_cid);
        let forest = helper.load_forest(forest_cid).await.unwrap();
        let (forest_cid, private_ref) = helper.init(forest, empty_key).await;
        let forest = helper.load_forest(forest_cid).await.unwrap();
        let root_dir = helper.get_root_dir(forest.to_owned(), private_ref.to_owned()).await.unwrap();
        let (new_cid, _) = helper.write_file(forest.to_owned(), root_dir.to_owned(), &["root".into(), "hello".into(), "world.txt".into()], b"hello, world!".to_vec()).await;
        let forest = helper.load_forest(new_cid).await.unwrap();
        let ls_result = helper.ls_files(forest.to_owned(), root_dir.to_owned(), &["root".into()]).await;
        println!("ls: {:?}", ls_result);
        let (new_cid, private_ref) = helper.mkdir(forest.to_owned(), root_dir.to_owned(), &["root".into(), "hi".into()]).await;
        let forest = helper.load_forest(new_cid).await.unwrap();
        let ls_result = helper.ls_files(forest.to_owned(), root_dir.to_owned(), &["root".into()]).await;
        assert_eq!(ls_result.get(0).unwrap().0, "hello");
        assert_eq!(ls_result.get(1).unwrap().0, "hi");
        let content = helper.read_file(forest.to_owned(), root_dir.to_owned(), &["root".into(), "hello".into(), "world.txt".into()]).await.unwrap();
        assert_eq!(content, b"hello, world!".to_vec());
        let (new_cid, private_ref) = helper.rm(forest, root_dir,  &["root".into(), "hello".into(), "world.txt".into()]).await;
        let forest = helper.load_forest(new_cid).await.unwrap();
        let root_dir = helper.get_root_dir(forest.to_owned(), private_ref.to_owned()).await.unwrap();
        let content = helper.read_file(forest.to_owned(), root_dir.to_owned(), &["root".into(), "hello".into(), "world.txt".into()]).await;
        assert_eq!(content, None);
        let private_ref_serialized = serde_json::to_string(&private_ref).unwrap();
        println!("private ref: \n{}", private_ref_serialized);
    }

}

//! This example shows how to add a directory to a private forest (also HAMT) which encrypts it.
//! It also shows how to retrieve encrypted nodes from the forest using `PrivateRef`s.

use chrono::Utc;
use libipld::Cid;
use rand::{thread_rng, rngs::ThreadRng};
use std::{rc::Rc};
use wnfs::{
    dagcbor,
    private::{PrivateForest, PrivateRef},
    BlockStore, Namefilter, PrivateDirectory, PrivateOpResult, Metadata,
};
use anyhow::Result;


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
        // Fetch and decrypt root directory from the private forest using provided private ref.
        forest
        .get(&private_ref, PrivateForest::resolve_lowest, &mut self.store)
        .await
        .unwrap().unwrap().as_dir()
    }

    pub async fn init(&mut self, forest: Rc<PrivateForest>) -> (Cid, PrivateRef) {
        // Create a new directory.
        let dir = Rc::new(PrivateDirectory::new(
            Namefilter::default(),
            Utc::now(),
            &mut self.rng,
        ));

        let PrivateOpResult { root_dir, forest, .. } = dir
            .mkdir(&["root".into()], true, Utc::now(), forest, &mut self.store,&mut self.rng)
            .await
            .unwrap();

        (self.update_forest(forest).await.unwrap(), root_dir.header.get_private_ref())
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


    pub fn synced_get_root_dir(&mut self, forest: Rc<PrivateForest>, private_ref: PrivateRef) -> Result<Rc<PrivateDirectory>, anyhow::Error>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.get_root_dir(forest, private_ref));
    }

    pub fn synced_init(&mut self, forest: Rc<PrivateForest>) -> (Cid, PrivateRef)
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.init(forest));
    }

    pub fn synced_write_file(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], content: Vec<u8>) -> (Cid, PrivateRef)
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.write_file(forest, root_dir, path_segments, content));
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
        let store = KVBlockStore::new(String::from("./tmp/test2"), IpldCodec::DagCbor);
        let blockstore = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(blockstore);
        let forest_cid = helper.create_private_forest().await.unwrap();
        println!("cid: {:?}", forest_cid);
        let forest = helper.load_forest(forest_cid).await.unwrap();
        let (forest_cid, private_ref) = helper.init(forest).await;
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

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
use crate::kvblockstore::KVBlockStore;

pub struct PrivateDirectoryHelper {
    pub store: KVBlockStore,
    rng: ThreadRng
}

// Single root (private ref) implementation of the wnfs private directory using KVBlockStore.
// TODO: we assumed all the write, mkdirs use same roots here. this could be done using prepend
// a root path to all path segments.
impl PrivateDirectoryHelper {
    pub fn new(db_path: String) -> Self {
        Self { 
            store: KVBlockStore::new(db_path),
            rng: thread_rng()
        }
    }

    pub async fn new_private_forest(&mut self) -> Cid {
        // Create the private forest (also HAMT), a map-like structure where files and directories are stored.
        let forest = Rc::new(PrivateForest::new());
        
        // Serialize the private forest to DAG CBOR.
        let cbor_bytes = dagcbor::async_encode(&forest, &mut self.store).await.unwrap();

        // Persist encoded private forest to the block store.
        self.store.put_serializable(&cbor_bytes).await.unwrap()
    }

    pub async fn load_forest(&mut self, forest_cid: Cid) -> Rc<PrivateForest> {
        // Fetch CBOR bytes of private forest from the blockstore.
        let cbor_bytes = self.store
            .get_deserializable::<Vec<u8>>(&forest_cid)
            .await
            .unwrap();

        // Decode private forest CBOR bytes.
        Rc::new(dagcbor::decode::<PrivateForest>(cbor_bytes.as_ref()).unwrap())
    }

    pub async fn update_forest(&mut self, hamt: Rc<PrivateForest>) -> Cid {
        // Serialize the private forest to DAG CBOR.
        let cbor_bytes = dagcbor::async_encode(&hamt, &mut self.store).await.unwrap();

        // Persist encoded private forest to the block store.
        self.store.put_serializable(&cbor_bytes).await.unwrap()
    }

    pub async fn get_root_dir(&mut self, forest: Rc<PrivateForest>, private_ref: PrivateRef) -> Rc<PrivateDirectory>{
        // Fetch and decrypt root directory from the private forest using provided private ref.
        forest
        .get(&private_ref, &mut self.store)
        .await
        .unwrap().unwrap().as_dir().unwrap()
    }

    pub async fn make_root_dir(&mut self, forest: Rc<PrivateForest>) -> (Cid, PrivateRef) {
        // Create a new directory.
        let dir = Rc::new(PrivateDirectory::new(
            Namefilter::default(),
            Utc::now(),
            &mut self.rng,
        ));

        let PrivateOpResult { root_dir, hamt, .. } = dir
            .mkdir(&["root".into()], true, Utc::now(), forest, &mut self.store,&mut self.rng)
            .await
            .unwrap();

        (self.update_forest(hamt).await, root_dir.header.get_private_ref().unwrap())
    }

    pub async fn write_file(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], content: Vec<u8>) -> Cid{
        let PrivateOpResult { hamt, .. } = root_dir
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
        self.update_forest(hamt).await

    }

    pub async fn read_file(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Vec<u8> {
        let PrivateOpResult { result, .. } = root_dir
            .read(path_segments, true, forest, &mut self.store)
            .await
            .unwrap();
        result
    }



    pub async fn mkdir(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Cid {
        let PrivateOpResult { hamt, .. } = root_dir
            .mkdir(path_segments, true, Utc::now(), forest, &mut self.store,&mut self.rng)
            .await
            .unwrap();

        self.update_forest(hamt).await
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
impl PrivateDirectoryHelper {
    pub fn synced_new_private_forest(&mut self) -> Cid
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.new_private_forest());
    }

    pub fn synced_load_forest(&mut self, forest_cid: Cid) -> Rc<PrivateForest>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.load_forest(forest_cid));
    }


    pub fn synced_get_root_dir(&mut self, forest: Rc<PrivateForest>, private_ref: PrivateRef) -> Rc<PrivateDirectory>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.get_root_dir(forest, private_ref));
    }

    pub fn synced_make_root_dir(&mut self, forest: Rc<PrivateForest>) -> (Cid, PrivateRef)
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.make_root_dir(forest));
    }

    pub fn synced_write_file(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], content: Vec<u8>) -> Cid
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.write_file(forest, root_dir, path_segments, content));
    }

    pub fn synced_read_file(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Vec<u8>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.read_file(forest, root_dir, path_segments));
    }

    pub fn synced_mkdir(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Cid
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.mkdir(forest, root_dir, path_segments));
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
    use crate::private::PrivateDirectoryHelper;


    #[async_std::test]
    async fn test_parse_path() {
        let path = "root/test.txt".to_string();
        let out = PrivateDirectoryHelper::parse_path(path);
        assert_eq!(out[0], "root".to_string());
        assert_eq!(out[1], "test.txt".to_string());
    }

    #[async_std::test]
    async fn iboverall() {
        let helper = &mut PrivateDirectoryHelper::new(String::from("./tmp/test"));
        let forest_cid = helper.new_private_forest().await;
        println!("cid: {:?}", forest_cid);
        let forest = helper.load_forest(forest_cid).await;
        let (forest_cid, private_ref) = helper.make_root_dir(forest).await;
        let forest = helper.load_forest(forest_cid).await;
        let root_dir = helper.get_root_dir(forest.to_owned(), private_ref.to_owned()).await;
        let new_cid = helper.write_file(forest.to_owned(), root_dir.to_owned(), &["root".into(), "hello".into(), "world.txt".into()], b"hello, world!".to_vec()).await;
        let forest = helper.load_forest(new_cid).await;
        let ls_result = helper.ls_files(forest.to_owned(), root_dir.to_owned(), &["root".into()]).await;
        println!("ls: {:?}", ls_result);
        let new_cid = helper.mkdir(forest.to_owned(), root_dir.to_owned(), &["root".into(), "hi".into()]).await;
        let forest = helper.load_forest(new_cid).await;
        let ls_result = helper.ls_files(forest.to_owned(), root_dir.to_owned(), &["root".into()]).await;
        assert_eq!(ls_result.get(0).unwrap().0, "hello");
        assert_eq!(ls_result.get(1).unwrap().0, "hi");
        let content = helper.read_file(forest.to_owned(), root_dir.to_owned(), &["root".into(), "hello".into(), "world.txt".into()]).await;
        assert_eq!(content, b"hello, world!".to_vec());
        let private_ref_serialized = serde_json::to_string(&private_ref).unwrap();
        println!("private ref: \n{}", private_ref_serialized);

    }

}

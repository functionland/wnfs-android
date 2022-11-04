//! This example shows how to add a directory to a private forest (also HAMT) which encrypts it.
//! It also shows how to retrieve encrypted nodes from the forest using `PrivateRef`s.

use chrono::Utc;
use libipld::Cid;
use rand::{thread_rng, RngCore};
use std::rc::Rc;
use wnfs::{
    dagcbor,
    private::{PrivateForest, PrivateRef},
    BlockStore, MemoryBlockStore, Namefilter, PrivateDirectory, PrivateOpResult,
};

pub struct PrivateDirectoryHelper {

}

impl PrivateDirectoryHelper {
    pub fn test_private_dir_synced() -> String
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(PrivateDirectoryHelper::test_private_dir());
    }

    async fn test_private_dir() -> String {
        // Create an in-memory block store.
        let store = &mut MemoryBlockStore::default();

        // Create a random number generator the private filesystem can use.
        let rng = &mut thread_rng();

        // Create a new private forest and get the cid to it.
        let (forest_cid, private_ref) = PrivateDirectoryHelper::get_forest_cid_and_private_ref(store, rng).await;

        // Fetch CBOR bytes of private forest from the blockstore.
        let cbor_bytes = store
            .get_deserializable::<Vec<u8>>(&forest_cid)
            .await
            .unwrap();

        // Decode private forest CBOR bytes.
        let forest = dagcbor::decode::<PrivateForest>(cbor_bytes.as_ref()).unwrap();

        // Fetch and decrypt a directory from the private forest using provided private ref.
        let dir = forest.get(&private_ref, store).await.unwrap();

        return format!("{:#?}", dir);
    }

    async fn get_forest_cid_and_private_ref<B, R>(store: &mut B, rng: &mut R) -> (Cid, PrivateRef)
    where
        B: BlockStore,
        R: RngCore,
    {
        // Create the private forest (also HAMT), a map-like structure where files and directories are stored.
        let forest = Rc::new(PrivateForest::new());

        // Create a new directory.
        let dir = Rc::new(PrivateDirectory::new(
            Namefilter::default(),
            Utc::now(),
            rng,
        ));

        // Add a /pictures/cats subdirectory.
        let PrivateOpResult {
            hamt: forest,
            root_dir,
            ..
        } = dir
            .mkdir(
                &["pictures".into(), "cats".into()],
                true,
                Utc::now(),
                forest,
                store,
                rng,
            )
            .await
            .unwrap();

        // Serialize the private forest to DAG CBOR.
        let cbor_bytes = dagcbor::async_encode(&forest, store).await.unwrap();

        // Persist encoded private forest to the block store.
        let forest_cid = store.put_serializable(&cbor_bytes).await.unwrap();

        // Private ref contains data and keys for fetching and decrypting the directory node in the private forest.
        let private_ref = root_dir.header.get_private_ref().unwrap();

        (forest_cid, private_ref)
    }
}
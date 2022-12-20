//! This example shows how to add a directory to a private forest (also HAMT) which encrypts it.
//! It also shows how to retrieve encrypted nodes from the forest using `PrivateRef`s.

use chrono::{Utc, prelude::*};
use libipld::Cid;
use rand::{thread_rng, rngs::ThreadRng};
use std::{
    rc::Rc, 
    fs::{File, OpenOptions}, 
    io::{Read, Write},
    os::unix::fs::MetadataExt
};
use wnfs::{
    dagcbor, Hasher, utils,
    private::{PrivateForest, PrivateRef, PrivateNode, Key},
    BlockStore, Namefilter, PrivateDirectory, PrivateOpResult, Metadata,
};
use anyhow::Result;
use log::{trace, Level};
use sha3::Sha3_256;
use futures::{Stream, StreamExt};


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

    pub async fn create_private_forest(&mut self) -> Result<Cid, String> {
        // Create the private forest (also HAMT), a map-like structure where files and directories are stored.
        let forest = Rc::new(PrivateForest::new());
        
        // Serialize the private forest to DAG CBOR.
        let cbor_bytes_res = dagcbor::async_encode(&forest, &mut self.store).await;
        if cbor_bytes_res.is_ok() {
            let cbor_bytes = cbor_bytes_res.ok().unwrap();

            // Persist encoded private forest to the block store.
            let store_res = self.store.put_serializable(&cbor_bytes).await;
            if store_res.is_ok() {
                Ok(store_res.ok().unwrap())
            } else {
                trace!("wnfsError occured in create_private_forest: {:?}", store_res.as_ref().err().unwrap());
                Err(store_res.err().unwrap().to_string())
            }
        } else {
            trace!("wnfsError occured in create_private_forest: {:?}", cbor_bytes_res.as_ref().err().unwrap());
            Err(cbor_bytes_res.err().unwrap().to_string())
        }
    }

    pub async fn load_forest(&mut self, forest_cid: Cid) -> Result<Rc<PrivateForest>, String> {
        // Fetch CBOR bytes of private forest from the blockstore.
        let cbor_bytes = self.store
            .get_deserializable::<Vec<u8>>(&forest_cid)
            .await;
        if cbor_bytes.is_ok() {
            let forest = dagcbor::decode::<PrivateForest>(cbor_bytes.ok().unwrap().as_ref());
            if forest.is_ok() {
                let loaded_forest = Rc::new(
                    forest
                    .ok()
                    .unwrap()
                );

                // Decode private forest CBOR bytes.
                Ok(loaded_forest)
            } else {
                trace!("wnfsError occured in load_forest: {:?}", forest.as_ref().err().unwrap());
                Err(forest.err().unwrap().to_string())
            }
        }else {
            trace!("wnfsError occured in load_forest: {:?}", cbor_bytes.as_ref().err().unwrap());
            Err(cbor_bytes.err().unwrap().to_string())
        }
    }

    pub async fn update_forest(&mut self, hamt: Rc<PrivateForest>) -> Result<Cid, String> {
        // Serialize the private forest to DAG CBOR.
        let cbor_bytes = dagcbor::async_encode(&hamt, &mut self.store).await;
        if cbor_bytes.is_ok() {
            // Persist encoded private forest to the block store.
            let res = self
                .store
                .put_serializable(
                &cbor_bytes
                    .ok()
                    .unwrap()
                )
                .await;
            if res.is_ok() {
                Ok(res.ok().unwrap())
            }   else {
                trace!("wnfsError occured in update_forest: {:?}", res.as_ref().err().unwrap().to_string());
                Err(res.err().unwrap().to_string())
            }
        } else {
            trace!("wnfsError occured in update_forest: {:?}", cbor_bytes.as_ref().err().unwrap().to_string());
            Err(cbor_bytes.err().unwrap().to_string())
        }
    }

    pub async fn get_root_dir(&mut self, forest: Rc<PrivateForest>, private_ref: PrivateRef) -> Result<Rc<PrivateDirectory>, String> {
        // Fetch and decrypt root directory from the private forest using provided private ref.
        let forest_res = forest
            .get(&private_ref, PrivateForest::resolve_lowest, &mut self.store)
            .await;
        if forest_res.is_ok() {
            let dir = forest_res.ok()
            .unwrap()
            .unwrap()
            .as_dir();
            if dir.is_ok() {
                Ok(dir.ok().unwrap())
            } else {
                trace!("wnfsError occured in get_root_dir: {:?}", dir.as_ref().err().unwrap().to_string());
                Err(dir.err().unwrap().to_string())
            }
        } else {
            trace!("wnfsError occured in get_root_dir: {:?}", forest_res.as_ref().err().unwrap().to_string());
            Err(forest_res.err().unwrap().to_string())
        }
    }

    pub async fn get_private_ref(&mut self, wnfs_key: Vec<u8>, forest_cid: Cid) -> Result<PrivateRef, String> {
        let ratchet_seed: [u8; 32] = Sha3_256::hash(&wnfs_key);
        let inumber: [u8; 32] = Sha3_256::hash(&ratchet_seed);
        let reloaded_private_ref = PrivateRef::with_seed(Namefilter::default(), ratchet_seed, inumber);
        

        let forest = self.load_forest(forest_cid)
        .await;
        if forest.is_ok() {
            let forest_unwrapped = forest
                .ok()
                .unwrap();
            let fetched_node = 
            forest_unwrapped
            .get(
                &reloaded_private_ref, 
                PrivateForest::resolve_lowest, 
                &mut self.store
            )
            .await;
            if fetched_node.is_ok() {

                let tmp = fetched_node
                    .ok()
                    .unwrap()
                    .unwrap()
                    .as_dir();
                if tmp.is_ok() {

                        let tmp_node = tmp.ok()
                            .unwrap()
                            .get_node(
                                &[], 
                                true, 
                                forest_unwrapped,  
                                &mut self.store
                            )
                            .await;
                        if tmp_node.is_ok() {
                            let latest_dir_res = tmp_node.ok()
                            .unwrap()
                            .result
                            .unwrap()
                            .as_dir();
                    
                            if latest_dir_res.is_ok() {
                                let latest_dir = latest_dir_res.ok().unwrap();

                                let private_ref = latest_dir.header.get_private_ref();

                                Ok(private_ref)
                            } else {
                                trace!("wnfsError in get_private_ref: latest_dir_res {:?}", latest_dir_res.as_ref().err().unwrap().to_string());
                                Err(latest_dir_res.err().unwrap().to_string())
                            }
                        } else {
                            trace!("wnfsError in get_private_ref: tmp_node {:?}", tmp_node.as_ref().err().unwrap().to_string());
                            Err(tmp_node.err().unwrap().to_string())
                        }
                } else {
                    trace!("wnfsError in get_private_ref: tmp {:?}", tmp.as_ref().err().unwrap().to_string());
                    Err(tmp.err().unwrap().to_string())
                }
            } else {
                trace!("wnfsError in get_private_ref fetched_node: {:?}", fetched_node.as_ref().err().unwrap().to_string());
                Err(fetched_node.err().unwrap().to_string())
            }
        } else {
            trace!("wnfsError in get_private_ref forest: {:?}", forest.as_ref().err().unwrap());
            Err(forest.err().unwrap())
        }
        
    }

    pub async fn init(&mut self, forest: Rc<PrivateForest>, wnfs_key: Vec<u8>) -> Result<(Cid, PrivateRef), String> {
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
        let root_dir = Rc::new(PrivateDirectory::with_seed( 
            Namefilter::default(), 
            Utc::now(), 
            ratchet_seed, 
            inumber, 
        ));

        let private_ref = root_dir.header.get_private_ref(); 
        let name = root_dir.header.get_saturated_name();
        
        let forest_res = forest 
            .put( 
                name, 
                &private_ref, 
                &PrivateNode::Dir(Rc::clone(&root_dir)), 
                &mut self.store, 
                &mut self.rng, 
            ) 
            .await;
        if forest_res.is_ok() {
        
            let init_private_ref = root_dir.header.get_private_ref();

            let update_res = self.update_forest(forest_res.ok().unwrap()).await;
            if update_res.is_ok() {
                Ok((update_res.ok().unwrap(), init_private_ref))
            }else {
                trace!("wnfsError occured in init: {:?}", update_res.as_ref().err().unwrap().to_string());
                Err(update_res.err().unwrap().to_string())
            }
        } else {
            trace!("wnfsError occured in init: {:?}", forest_res.as_ref().err().unwrap().to_string());
            Err(forest_res.err().unwrap().to_string())
        }
    }

    fn get_file_as_byte_vec(&mut self, filename: &String) -> Result<(Vec<u8>, i64), String> {
        let f = File::open(&filename);
        if f.is_ok() {
            let metadata_res = std::fs::metadata(&filename);
            if metadata_res.is_ok() {
                let metadata = metadata_res.ok().unwrap();
                let modification_time_seconds = metadata.mtime();

                let mut buffer = vec![0; metadata.len() as usize];
                f.ok().unwrap().read(&mut buffer).expect("buffer overflow");
                Ok((buffer, modification_time_seconds))
            } else {
                trace!("wnfsError in get_file_as_byte_vec, unable to read metadata: {:?}", metadata_res.err().unwrap());
                Err("wnfsError unable to read metadata".to_string())
            }
        } else {
            trace!("wnfsError in get_file_as_byte_vec, no file found: {:?}", f.err().unwrap());
            Err("wnfsError no file found".to_string())
        }
        
    }

    pub async fn write_file_from_path(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], filename: &String) -> Result<(Cid, PrivateRef), String> {
        let content: Vec<u8>;
        let modification_time_seconds: i64;
        let try_content = self.get_file_as_byte_vec(filename);
        if try_content.is_ok() {
            (content, modification_time_seconds) = try_content.ok().unwrap();
            let writefile_res = self.write_file(forest, root_dir, path_segments, content, modification_time_seconds).await;
            if writefile_res.is_ok() {
                Ok(writefile_res.ok().unwrap())
            }else{
                trace!("wnfsError in write_file_from_path: {:?}", writefile_res.as_ref().err().unwrap());
                Err(writefile_res.err().unwrap())
            }
        } else {
            trace!("wnfsError in write_file_from_path: {:?}", try_content.as_ref().err().unwrap());
            Err(try_content.err().unwrap())
        }
        
    }

    fn write_byte_vec_to_file(&mut self, filename: &String, file_content: Vec<u8>) -> Result<bool, String> {
        trace!("wnfs11 **********************write_byte_vec_to_file started**************filename={:?}", filename);
        trace!("wnfs11 **********************write_byte_vec_to_file started**************file_content={:?}", file_content);
        let file = File::create(filename);
        if file.is_ok() {
            let mut file_handler = file.ok().unwrap();
            trace!("wnfs11 **********************write_byte_vec_to_file write created**************");
            let write_res = file_handler
            .write_all(&file_content);
            if write_res.is_ok() {
                Ok(true)
            } else {
                trace!("wnfsError occured in write_byte_vec_to_file on write_res {:?}", write_res.as_ref().err().unwrap().to_string());
                Err(write_res.err().unwrap().to_string())
            }
        } else {
            trace!("wnfsError occured in write_byte_vec_to_file on file {:?}", file.as_ref().err().unwrap().to_string());
            Err(file.err().unwrap().to_string())
        }
        
    }

    pub async fn write_file(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], content: Vec<u8>, modification_time_seconds: i64) -> Result<(Cid, PrivateRef), String> {
        let mut modification_time_utc: DateTime<Utc>= Utc::now();
        if modification_time_seconds > 0 {
            let naive_datetime = NaiveDateTime::from_timestamp_opt(modification_time_seconds, 0).unwrap();
            modification_time_utc = DateTime::from_utc(naive_datetime, Utc);
        }
        let write_res = root_dir
            .write(
                path_segments,
                true,
                modification_time_utc,
                content,
                forest,
                &mut self.store,
                &mut self.rng,
            )
            .await;
            if write_res.is_ok() {
                let PrivateOpResult { forest, root_dir, .. } = write_res.ok().unwrap();
                let config = (self.update_forest(forest).await.unwrap(), root_dir.header.get_private_ref());
                Ok(config)
            } else {
                trace!("wnfsError in write_file: {:?}", write_res.as_ref().err().unwrap().to_string());
                Err(write_res.err().unwrap().to_string())
            }
    }

    pub async fn read_filestream_to_path(&mut self, local_filename: &String, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], index: usize) -> Result<bool, String> {
        //let mut stream_content: Vec<u8> = vec![];
        let local_file = File::create(local_filename);
        if local_file.is_ok() {
            let mut local_file_handler = local_file.ok().unwrap();
        
            let private_node_result = root_dir.get_node(
                path_segments
                , true
                , forest
                , &mut self.store
            ).await;
            if private_node_result.is_ok() {
                let PrivateOpResult {result, forest, ..} = private_node_result.ok().unwrap();
                if result.is_some() {
                    let private_node = result.unwrap();
                    let is_file = private_node.is_file();
                    if is_file {
                        let file_res = private_node.as_file();
                        if file_res.is_ok() {
                            let file = file_res.ok().unwrap();
                            let mut stream = file.stream_content(
                                index
                                , &forest
                                , &mut self.store
                            );
                            while let Some(block) = stream.next().await {
                                let write_result = local_file_handler.write_all(&block.unwrap());
                                if write_result.is_err() {
                                    trace!("wnfsError occured in read_filestream_to_path on write_result: {:?}", write_result.as_ref().err().unwrap().to_string());
                                }
                                //stream_content.extend_from_slice(&block.unwrap());
                            }
                            Ok(true)
                        } else {
                            trace!("wnfsError occured in read_filestream_to_path on file_res: {:?}", file_res.as_ref().err().unwrap().to_string());
                            Err(file_res.err().unwrap().to_string())
                        }
                    } else {
                        trace!("wnfsError occured in read_filestream_to_path on is_file");
                        Err("wnfsError occured in read_filestream_to_path on is_file".to_string())
                    }
                } else {
                    trace!("wnfsError occured in read_filestream_to_path on result");
                    Err("wnfsError occured in read_filestream_to_path on result".to_string())
                }
            } else {
                trace!("wnfsError occured in read_filestream_to_path on private_node_result: {:?}", private_node_result.as_ref().err().unwrap().to_string());
                Err(private_node_result.err().unwrap().to_string())
            }
        } else {
            trace!("wnfsError occured in read_filestream_to_path on local_file {:?}", local_file.as_ref().err().unwrap().to_string());
            Err(local_file.err().unwrap().to_string())
        }
    }

    pub async fn read_file_to_path(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], filename: &String) -> Result<String, String> {
        let file_content_res = self.read_file(forest, root_dir, path_segments).await;
        if file_content_res.is_ok() {
            let res = self.write_byte_vec_to_file(filename, file_content_res.ok().unwrap());
            if res.is_ok() {
                Ok(filename.to_string())
            } else {
                trace!("wnfsError occured in read_file_to_path on res: {:?}", res.as_ref().err().unwrap().to_string());
                Err(res.err().unwrap().to_string())
            }
        } else {
            trace!("wnfsError occured in read_file_to_path on file_content_res: {:?}", file_content_res.as_ref().err().unwrap().to_string());
            Err(file_content_res.err().unwrap().to_string())
        }
    }

    pub async fn read_file(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Result<Vec<u8>, String> {
        let res = root_dir
            .read(path_segments, true, forest, &mut self.store)
            .await;
        if res.is_ok() {
            let PrivateOpResult { result, .. } = res.ok().unwrap();
            Ok(result)
        } else {
            trace!("wnfsError occured in read_file: {:?} ", res.as_ref().err().unwrap());
            Err(res.err().unwrap().to_string())
        }
    }


    pub async fn mkdir(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Result<(Cid, PrivateRef), String> {
        let res = root_dir
            .mkdir(path_segments, true, Utc::now(), forest, &mut self.store,&mut self.rng)
            .await;
        if res.is_ok() {
            let PrivateOpResult { forest, root_dir, .. } = res .unwrap();

            let update_res = self.update_forest(forest).await;
            if update_res.is_ok() {
                Ok((update_res.ok().unwrap(), root_dir.header.get_private_ref()))
            } else {
                trace!("wnfsError occured in mkdir: {:?}", update_res.as_ref().err().unwrap());
                Err(update_res.err().unwrap())
            }
        } else {
            trace!("wnfsError occured in mkdir: {:?}", res.as_ref().err().unwrap());
            Err(res.err().unwrap().to_string())
        }
    }


    pub async fn rm(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Result<(Cid, PrivateRef), String> {
        let result = root_dir
            .rm(path_segments, true, forest, &mut self.store,&mut self.rng)
            .await;
        if result.is_ok() {
            let PrivateOpResult { forest, root_dir, .. } = result
                .ok()
                .unwrap();
            let update_res = self.update_forest(forest).await;
            if update_res.is_ok() {
                 Ok((update_res.ok().unwrap(), root_dir.header.get_private_ref()))
            } else {
                trace!("wnfsError occured in rm update_res: {:?}", update_res.as_ref().err().unwrap());
                Err(update_res.err().unwrap().to_string())
            }
        } else {
            trace!("wnfsError occured in rm result: {:?}", result.as_ref().err().unwrap());
            Err(result.err().unwrap().to_string())
        }
        
    }

    pub async fn mv(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, source_path_segments: &[String], target_path_segments: &[String]) -> Result<(Cid, PrivateRef), String> {
        let mv_result = root_dir
            .basic_mv(
                source_path_segments,
                target_path_segments,
                true,
                Utc::now(),
                forest,
                &mut self.store,
                &mut self.rng,
            )
            .await;
            if mv_result.is_ok() {
                let PrivateOpResult {
                    root_dir, forest, ..
                } = mv_result.ok().unwrap();

                let update_res = self.update_forest(forest).await;
                if update_res.is_ok() {
                     Ok((update_res.ok().unwrap(), root_dir.header.get_private_ref()))
                } else {
                    trace!("wnfsError occured in mv update_res: {:?}", update_res.as_ref().err().unwrap());
                    Err(update_res.err().unwrap().to_string())
                }
            } else {
                trace!("wnfsError occured in mv mv_result: {:?}", mv_result.as_ref().err().unwrap());
                Err(mv_result.err().unwrap().to_string())
            }
    }

    pub async fn cp(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, source_path_segments: &[String], target_path_segments: &[String]) -> Result<(Cid, PrivateRef), String> {
        let cp_result = root_dir
                .cp(
                    source_path_segments,
                    target_path_segments,
                    true,
                    Utc::now(),
                    forest,
                    &mut self.store,
                    &mut self.rng
                 )
                 .await;
            if cp_result.is_ok() {
                let PrivateOpResult { forest, root_dir, .. } 
                = cp_result.ok().unwrap();

                let update_res = self.update_forest(forest).await;
                if update_res.is_ok() {
                     Ok((update_res.ok().unwrap(), root_dir.header.get_private_ref()))
                } else {
                    trace!("wnfsError occured in cp update_res: {:?}", update_res.as_ref().err().unwrap());
                    Err(update_res.err().unwrap().to_string())
                }
            } else {
                trace!("wnfsError occured in cp cp_result: {:?}", cp_result.as_ref().err().unwrap());
                Err(cp_result.err().unwrap().to_string())
            }
    }

    pub async fn ls_files(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Result<Vec<(String, Metadata)>, String> {

        let res = root_dir
            .ls(path_segments, true, forest, &mut self.store)
            .await;
        if res.is_ok() {
            let PrivateOpResult { result, .. } = res.ok().unwrap();
            Ok(result)
        } else {
            trace!("wnfsError occured in ls_files: {:?}", res.as_ref().err().unwrap().to_string());
            Err(res.err().unwrap().to_string())
        }
    }

}

// Implement synced version of the library for using in android jni.
impl<'a> PrivateDirectoryHelper<'a> {
    pub fn synced_create_private_forest(&mut self) -> Result<Cid, String>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.create_private_forest());
    }

    pub fn synced_load_forest(&mut self, forest_cid: Cid) -> Result<Rc<PrivateForest>, String>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.load_forest(forest_cid));
    }

    pub fn synced_get_private_ref(&mut self, wnfs_key: Vec<u8>, forest_cid: Cid) -> Result<PrivateRef, String>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.get_private_ref(wnfs_key, forest_cid));
    }


    pub fn synced_get_root_dir(&mut self, forest: Rc<PrivateForest>, private_ref: PrivateRef) -> Result<Rc<PrivateDirectory>, String>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.get_root_dir(forest, private_ref));
    }

    pub fn synced_init(&mut self, forest: Rc<PrivateForest>, wnfs_key: Vec<u8>) -> Result<(Cid, PrivateRef), String>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.init(forest, wnfs_key));
    }

    pub fn synced_write_file_from_path(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], filename: &String) -> Result<(Cid, PrivateRef), String>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.write_file_from_path(forest, root_dir, path_segments, filename));
    }

    pub fn synced_write_file(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], content: Vec<u8>, modification_time_seconds: i64) -> Result<(Cid, PrivateRef), String>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.write_file(forest, root_dir, path_segments, content, modification_time_seconds));
    }

    pub fn synced_read_file_to_path(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], filename: &String) -> Result<String, String>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.read_file_to_path(forest, root_dir, path_segments, filename));
    }

    pub fn synced_read_file(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Result<Vec<u8>, String>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.read_file(forest, root_dir, path_segments));
    }

    pub fn synced_read_filestream_to_path(&mut self, local_filename: &String, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String], index: usize) -> Result<bool, String> {
        let runtime =
        tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.read_filestream_to_path(local_filename, forest, root_dir, path_segments, index));
    }

    pub fn synced_mkdir(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Result<(Cid, PrivateRef), String>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.mkdir(forest, root_dir, path_segments));
    }

    pub fn synced_mv(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, source_path_segments: &[String], target_path_segments: &[String]) -> Result<(Cid, PrivateRef), String> {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.mv(forest, root_dir, source_path_segments, target_path_segments));
    }

    pub fn synced_cp(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, source_path_segments: &[String], target_path_segments: &[String]) -> Result<(Cid, PrivateRef), String> {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.cp(forest, root_dir, source_path_segments, target_path_segments));
    }

    pub fn synced_rm(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Result<(Cid, PrivateRef), String>
    {
        let runtime =
            tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        return runtime.block_on(self.rm(forest, root_dir, path_segments));
    }

    pub fn synced_ls_files(&mut self, forest: Rc<PrivateForest>, root_dir: Rc<PrivateDirectory>, path_segments: &[String]) -> Result<Vec<(String, Metadata)>, String>
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
        let (new_cid, _) = helper.write_file(forest.to_owned(), root_dir.to_owned(), &["root".into(), "hello".into(), "world.txt".into()], b"hello, world!".to_vec(), 0).await;
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

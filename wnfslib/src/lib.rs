// #[cfg(target_os = "android")]
// #[allow(non_snake_case)]
// #[allow(unused_imports)]

pub mod android {
    extern crate jni;

    use jni::objects::{JClass, JObject, JString, JValue};
    use jni::signature::JavaType;
    use jni::sys::{jbyteArray, jobject, jstring};
    use jni::JNIEnv;
    use libipld::Cid;
    use wnfs::private::AccessKey;
    use wnfs::common::Metadata;
    use log::{trace, Level};
    extern crate android_logger;
    use android_logger::Config;
    use anyhow::Result;
    use wnfsutils::blockstore::{FFIFriendlyBlockStore, FFIStore};
    use wnfsutils::private_forest::PrivateDirectoryHelper;

    struct JNIStore<'a> {
        env: JNIEnv<'a>,
        fula_client: JObject<'a>,
    }

    impl<'a> JNIStore<'a> {
        fn new(env: JNIEnv<'a>, fula_client: JObject<'a>) -> Self {
            Self { env: env, fula_client: fula_client }
        }
    }

    impl<'a> FFIStore<'a> for JNIStore<'a> {
        /// Retrieves an array of bytes from the block store with given CID.
        fn get_block(&self, cid: Vec<u8>) -> Result<Vec<u8>> {
            trace!("**********************get_block started**************");
            trace!("**********************get_block started**************");
            trace!("**********************get_block bytes={:?}", &cid);
            let get_fn = self
                .env
                .get_method_id(self.fula_client, "get", "([B)[B")
                .unwrap();

            let cid_jbyte_array = vec_to_jbyte_array(self.env, cid);
            unsafe{
                let data_jbyte_array = self
                    .env
                    .call_method_unchecked(
                        self.fula_client,
                        get_fn,
                        JavaType::Object(String::from("[B")),
                        &[JValue::from(cid_jbyte_array)],
                    )
                    .unwrap()
                    .l()
                    .unwrap();

                let data = jbyte_array_to_vec(self.env, data_jbyte_array.into_inner());
                trace!("**********************get_block finished**************");
                Ok(data)
            }
        }

        /// Stores an array of bytes in the block store.
        fn put_block(&self, cid: Vec<u8>, bytes: Vec<u8>) -> Result<()> {
            trace!("**********************put_block started**************");
            trace!("**********************put_block cid={:?}", &cid);
            trace!("**********************put_block bytes={:?}", &bytes);
            let put_fn = self
                .env
                .get_method_id(self.fula_client, "put", "([B[B)[B")
                .unwrap();
            trace!("**********************put_block put_fn done**************");
            let data_jbyte_array = vec_to_jbyte_array(self.env, bytes);
            trace!("**********************put_block data_jbyte_array done**************");
            trace!(
                "**********************put_block LVALUE_data_jbyte_array={:?}",
                &JValue::from(data_jbyte_array)
            );
            let cid_jbyte_array = vec_to_jbyte_array(self.env, cid);
            trace!("**********************put_block cid_jbyte_array done**************");
            trace!(
                "**********************put_block LVALUE_cid_jbyte_array={:?}",
                &JValue::from(cid_jbyte_array)
            );
            unsafe{
                let cid_jbyte_array = self
                    .env
                    .call_method_unchecked(
                        self.fula_client,
                        put_fn,
                        JavaType::Object(String::from("[B")),
                        &[JValue::from(cid_jbyte_array), JValue::from(data_jbyte_array)],
                    )
                    .unwrap_or_else(|_err: jni::errors::Error| {
                        trace!("**********************put_block first unwrap error**************");
                        panic!("HERE1: {}", _err)
                    })
                    .l()
                    .unwrap_or_else(|_err: jni::errors::Error| {
                        trace!("**********************put_block second unwrap error**************");
                        panic!("HERE2: {}", _err)
                    });
                trace!("**********************put_block cid_jbyte_array done**************");
                let _ = jbyte_array_to_vec(self.env, cid_jbyte_array.into_inner());
                trace!("**********************put_block finished**************");
                Ok(())
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_initRustLogger(_: JNIEnv, _: JClass) {
        android_logger::init_once(Config::default().with_min_level(Level::Trace));
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_createPrivateForestNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
    ) -> jstring {
        trace!("**********************createPrivateForest started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);
        trace!("**********************createPrivateForest finished**************");
        let private_forest = helper.synced_create_private_forest();
        if private_forest.is_ok() {
            serialize_cid(env, private_forest.ok().unwrap()).into_inner()
        } else {
            env
            .new_string("")
            .expect("Failed to serialize result")
            .into_inner()
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_getAccessKeyNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_wnfs_key: jbyteArray,
        jni_cid: JString,
    ) -> jstring {
        trace!("**********************getAccessKeyNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);
        let wnfs_key: Vec<u8> = jbyte_array_to_vec(env, jni_wnfs_key);
        let forest_cid = deserialize_cid(env, jni_cid);
        let forest_res = &mut helper.synced_load_forest(forest_cid);
        if forest_res.is_ok() {
            let forest = &mut forest_res.ok().unwrap();
        }
        let access_key = helper.synced_get_access_key(wnfs_key, forest_cid);
        trace!("**********************getAccessKeyNative finished**************");
        if access_key.is_ok() {
            return serialize_access_key(env, access_key.ok().unwrap()).into_inner();
        } else {
            env
            .new_string("")
            .expect("Failed to serialize result")
            .into_inner()
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_createRootDirNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_wnfs_key: jbyteArray,
    ) -> jobject {
        trace!("**********************createRootDirNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);
        let forest_cid = deserialize_cid(env, jni_cid);
        trace!("cid: {}", forest_cid);
        let forest_res = &mut helper.synced_load_forest(forest_cid);
        if forest_res.is_ok() {
            let forest = &mut forest_res.ok().unwrap();
            let wnfs_key: Vec<u8> = jbyte_array_to_vec(env, jni_wnfs_key);
            let init_res = helper.synced_init(forest, wnfs_key);
            if init_res.is_ok() {
                let (cid, access_key) = init_res.ok().unwrap();
                trace!("pref: {:?}", access_key);
                trace!("**********************createRootDirNative finished**************");
                unsafe{
                    return serialize_config(env, cid, access_key)
                }
            } else {
                let msg = init_res.err().unwrap();
                trace!("wnfsError in Java_land_fx_wnfslib_Fs_createRootDirNative: {:?}", msg);
                return JObject::null().into_inner();
            }
        } else {
            let msg = &mut forest_res.err().unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_createRootDirNative: {:?}", msg);
            return JObject::null().into_inner();
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_writeFileFromPathNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_access_key: JString,
        jni_path_segments: JString,
        jni_filename: JString,
    ) -> jobject {
        trace!("**********************writeFileFromPathNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);

        let cid = deserialize_cid(env, jni_cid);
        let access_key = deserialize_access_key(env, jni_access_key);
        let old_access_key = access_key.to_owned();
        let old_cid = cid.to_owned();

        let forest_res = &mut helper.synced_load_forest(cid);
        if forest_res.is_ok() {
            let forest = &mut forest_res.ok().unwrap();
            let root_dir_res = helper
                .synced_get_root_dir(forest, access_key);
            if root_dir_res.is_ok() {
                let root_dir = &mut root_dir_res.ok().unwrap();
                let path_segments = prepare_path_segments(env, jni_path_segments);

                let filename: String = env
                    .get_string(jni_filename)
                    .expect("Failed to parse input path segments")
                    .into();

                let write_file_result = 
                    helper.synced_write_file_from_path(forest, root_dir, &path_segments, &filename);
                    trace!("**********************writeFileFromPathNative finished**************");
                if write_file_result.is_ok() {
                    let (cid, access_key) = write_file_result.ok().unwrap();
                    unsafe{
                        return serialize_config(env, cid, access_key);
                    }
                } else {
                    let msg = write_file_result.err().unwrap();
                    trace!("wnfsError in Java_land_fx_wnfslib_Fs_writeFileFromPathNative: {:?}", msg);
                    return JObject::null().into_inner();
                }
            } else {
                let msg = root_dir_res.err().unwrap();
                trace!("wnfsError in Java_land_fx_wnfslib_Fs_writeFileFromPathNative: {:?}", msg);
                return JObject::null().into_inner();
            }
        } else {
            let msg = &mut forest_res.err().unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_writeFileFromPathNative: {:?}", msg);
            return JObject::null().into_inner();
        }
        
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_readFilestreamToPathNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_access_key: JString,
        jni_path_segments: JString,
        jni_filename: JString,
    ) -> jstring {
        trace!("wnfs11 **********************readFilestreamToPathNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);

        let cid = deserialize_cid(env, jni_cid);
        let access_key = deserialize_access_key(env, jni_access_key);

        let forest = &mut helper.synced_load_forest(cid).unwrap();
        let root_dir = &mut helper
            .synced_get_root_dir(forest, access_key)
            .unwrap();
        let path_segments = prepare_path_segments(env, jni_path_segments);
        let filename: String = env
            .get_string(jni_filename)
            .expect("Failed to parse input path segments")
            .into();
        trace!("wnfs11 **********************readFilestreamToPathNative filename created**************");
        let result = helper.synced_read_filestream_to_path(&filename, forest, root_dir, &path_segments, 0);
        trace!("wnfs11 **********************readFilestreamToPathNative finished**************");
        if result.is_ok() {
            let res = result.ok().unwrap();
            env
                .new_string(filename)
                .expect("Failed to serialize result")
                .into_inner()
        } else {
            trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_readFilestreamToPathNative on result: {:?}", result.err().unwrap());
            env
                .new_string("".to_string())
                .expect("Failed to serialize result")
                .into_inner()
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_readFileToPathNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_access_key: JString,
        jni_path_segments: JString,
        jni_filename: JString,
    ) -> jstring {
        trace!("wnfs11 **********************readFileToPathNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);

        let cid = deserialize_cid(env, jni_cid);
        let access_key = deserialize_access_key(env, jni_access_key);

        let forest = &mut helper.synced_load_forest(cid).unwrap();
        let root_dir = &mut helper
            .synced_get_root_dir(forest, access_key)
            .unwrap();
        let path_segments = prepare_path_segments(env, jni_path_segments);
        let filename: String = env
            .get_string(jni_filename)
            .expect("Failed to parse input path segments")
            .into();
        trace!("wnfs11 **********************readFileToPathNative filename created**************");
        let result = helper.synced_read_file_to_path(forest, root_dir, &path_segments, &filename);
        trace!("wnfs11 **********************readFileToPathNative finished**************");
        if result.is_ok() {
            let res = result.ok().unwrap();
            env
                .new_string(filename)
                .expect("Failed to serialize result")
                .into_inner()
        } else {
            trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_readFileToPathNative {:?}", result.err().unwrap());
            env
                .new_string("".to_string())
                .expect("Failed to serialize result")
                .into_inner()
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_writeFileNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_access_key: JString,
        jni_path_segments: JString,
        jni_content: jbyteArray,
    ) -> jobject {
        trace!("**********************writeFileNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);

        let cid = deserialize_cid(env, jni_cid);
        let access_key = deserialize_access_key(env, jni_access_key);

        let forest = &mut helper.synced_load_forest(cid).unwrap();
        let root_dir = &mut helper
            .synced_get_root_dir(forest, access_key)
            .unwrap();
        let path_segments = prepare_path_segments(env, jni_path_segments);
        let content = jbyte_array_to_vec(env, jni_content);
        //let (cid, access_key) =
        let write_file_res = 
            helper.synced_write_file(forest, root_dir, &path_segments, content, 0);
        trace!("**********************writeFileNative finished**************");
        if write_file_res.is_ok() {
            let (cid, access_key) = write_file_res.ok().unwrap();
            unsafe{
                let config: jobject = serialize_config(env, cid, access_key);
                return config;
            }
        } else {
            let msg = write_file_res
                .err()
                .unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_writeFileNative: {:?}", msg);
            return JObject::null().into_inner();
            
        }
        
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_readFileNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_access_key: JString,
        jni_path_segments: JString,
    ) -> jbyteArray {
        trace!("**********************readFileNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);

        let cid = deserialize_cid(env, jni_cid);
        let access_key = deserialize_access_key(env, jni_access_key);

        let forest = &mut helper.synced_load_forest(cid).unwrap();
        let root_dir = &mut helper
            .synced_get_root_dir(forest, access_key)
            .unwrap();
        let path_segments = prepare_path_segments(env, jni_path_segments);
        trace!("**********************readFileNative finished**************");
        let result = helper.synced_read_file(forest, root_dir, &path_segments);
        if result.is_err() {
            let empty_vec: Vec<u8> = Vec::new();
            return vec_to_jbyte_array(
                env,
                empty_vec,
            );
        }
        vec_to_jbyte_array(
            env,
            result.ok().unwrap(),
        )
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_mkdirNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_access_key: JString,
        jni_path_segments: JString,
    ) -> jobject {
        trace!("**********************mkDirNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);

        let cid = deserialize_cid(env, jni_cid);
        let access_key = deserialize_access_key(env, jni_access_key);

        let forest_res = &mut helper.synced_load_forest(cid);
        if forest_res.is_ok() {
            let forest = &mut forest_res.ok().unwrap();
            let root_dir_res = helper
                .synced_get_root_dir(forest, access_key);
                if root_dir_res.is_ok() {
                    let root_dir = &mut root_dir_res.ok().unwrap();
                    let path_segments = prepare_path_segments(env, jni_path_segments);
                    let mkdir_res = helper.synced_mkdir(forest,root_dir, &path_segments);
                    if mkdir_res.is_ok() {
                        let (cid, access_key) = mkdir_res.ok().unwrap();
                        trace!("**********************mkDirNative finished**************");
                        unsafe{return serialize_config(env, cid, access_key)}
                    } else {
                        let msg = mkdir_res
                            .err()
                            .unwrap();
                        trace!("wnfsError in Java_land_fx_wnfslib_Fs_mkdirNative: {:?}", msg);
                        return JObject::null().into_inner();
                    }
                } else {
                    let msg = root_dir_res
                        .err()
                        .unwrap();
                    trace!("wnfsError in Java_land_fx_wnfslib_Fs_mkdirNative: {:?}", msg);
                    return JObject::null().into_inner();
                }
        } else {
            let msg = &mut forest_res
                .err()
                .unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_mkdirNative: {:?}", msg);
            return JObject::null().into_inner();
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_mvNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_access_key: JString,
        jni_source_path_segments: JString,
        jni_target_path_segments: JString,
    ) -> jobject {
        trace!("**********************mvNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);

        let cid = deserialize_cid(env, jni_cid);
        let access_key = deserialize_access_key(env, jni_access_key);

        let forest = &mut helper.synced_load_forest(cid).unwrap();
        let root_dir = &mut helper
            .synced_get_root_dir(forest, access_key)
            .unwrap();
        let source_path_segments = prepare_path_segments(env, jni_source_path_segments);
        let target_path_segments = prepare_path_segments(env, jni_target_path_segments);
        let result = helper.synced_mv(forest, root_dir, &source_path_segments, &target_path_segments);
        trace!("**********************mvNative finished**************");
        if result.is_ok() {
            let (cid, access_key) = result.ok().unwrap();
            unsafe{
                return serialize_config(env, cid, access_key);
            }
        }else {
            trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_mvNative: {:?}", result.err().unwrap());
            return JObject::null().into_inner();
        }
        
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_cpNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_access_key: JString,
        jni_source_path_segments: JString,
        jni_target_path_segments: JString,
    ) -> jobject {
        trace!("**********************cpNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);

        let cid = deserialize_cid(env, jni_cid);
        let access_key = deserialize_access_key(env, jni_access_key);

        let forest = &mut helper.synced_load_forest(cid).unwrap();
        let root_dir = &mut helper
            .synced_get_root_dir(forest, access_key)
            .unwrap();
        let source_path_segments = prepare_path_segments(env, jni_source_path_segments);
        let target_path_segments = prepare_path_segments(env, jni_target_path_segments);
        let result = helper.synced_cp(forest, root_dir, &source_path_segments, &target_path_segments);
        trace!("**********************mvNative finished**************");
        if result.is_ok() {
            let (cid, access_key) = result.ok().unwrap();
            unsafe{
                return serialize_config(env, cid, access_key);
            }
        }else {
            trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_cpNative: {:?}", result.err().unwrap());
            return JObject::null().into_inner();
        }
        
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_rmNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_access_key: JString,
        jni_path_segments: JString,
    ) -> jobject {
        trace!("**********************rmNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);

        let cid = deserialize_cid(env, jni_cid);
        let access_key = deserialize_access_key(env, jni_access_key);

        let forest = &mut helper.synced_load_forest(cid).unwrap();
        let root_dir = &mut helper
            .synced_get_root_dir(forest, access_key)
            .unwrap();
        let path_segments = prepare_path_segments(env, jni_path_segments);
        let result = helper.synced_rm(forest, root_dir, &path_segments);
        trace!("**********************rmNative finished**************");
        if result.is_ok() {
            let (cid, access_key) = result.ok().unwrap();
            unsafe{
                return serialize_config(env, cid, access_key);
            }
        }else {
            trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_rmNative: {:?}", result.err().unwrap());
            return JObject::null().into_inner();
        }
        
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_lsNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_access_key: JString,
        jni_path_segments: JString,
    ) -> jbyteArray {
        trace!("**********************lsNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);

        let cid = deserialize_cid(env, jni_cid);
        let access_key = deserialize_access_key(env, jni_access_key);

        let forest_res = &mut helper.synced_load_forest(cid);
        if forest_res.is_ok() {
            let forest = &mut forest_res.ok().unwrap();
            let root_dir_res = helper
                .synced_get_root_dir(forest, access_key);
            if root_dir_res.is_ok() {
                let root_dir = &mut root_dir_res.ok().unwrap();
                let path_segments = prepare_path_segments(env, jni_path_segments);
                let ls_res = helper.synced_ls_files(forest, root_dir, &path_segments);
                if ls_res.is_ok() {
                    let output =
                        prepare_ls_output(env, ls_res.ok().unwrap());
                    trace!("**********************lsNative finished**************");
                    if output.is_ok() {
                        let res = output.ok().unwrap();
                        return vec_to_jbyte_array(
                            env,
                            res
                        );
                    } else {
                        trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_lsNative output: {:?}", output.err().unwrap().to_string());
                        let empty_bytes: Vec<u8> = vec![0];
                        return vec_to_jbyte_array(
                            env,
                            empty_bytes
                        );
                    }
                } else {
                    trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_lsNative ls_res: {:?}", ls_res.err().unwrap().to_string());
                    let empty_bytes: Vec<u8> = vec![0];
                    return vec_to_jbyte_array(
                        env,
                        empty_bytes
                    );
                }
            } else {
                trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_lsNative root_dir_res: {:?}", root_dir_res.err().unwrap().to_string());
                let empty_bytes: Vec<u8> = vec![0];
                return vec_to_jbyte_array(
                    env,
                    empty_bytes
                );
            }
        } else {
            trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_lsNative forest_res: {:?}", forest_res.err().unwrap().to_string());
            let empty_bytes: Vec<u8> = vec![0];
            return vec_to_jbyte_array(
                env,
                empty_bytes
            );
        }
    }

    pub unsafe fn serialize_config(env: JNIEnv, cid: Cid, access_key: AccessKey) -> jobject {
        trace!("**********************serialize_config started**************");
        let config_cls = env.find_class("land/fx/wnfslib/Config").unwrap();
        //let  handler_class = reinterpret_cast<jclass>(env.new_global_ref(config_cls));
        trace!("**********************serialize_config config_cls set**************");
        let create_config_fn_res = env
            .get_static_method_id(
                config_cls,
                "create",
                "(Ljava/lang/String;Ljava/lang/String;)Lland/fx/wnfslib/Config;",
            );
        if create_config_fn_res.is_ok() {
            let create_config_fn = create_config_fn_res.ok().unwrap();

            trace!("**********************serialize_config create_config_fn set**************");
            let cid = serialize_cid(env, cid);
            let access_key = serialize_access_key(env, access_key);
            trace!("**********************serialize_config almost finished**************");
            let config_res = env
                .call_static_method_unchecked(
                    config_cls,
                    create_config_fn,
                    JavaType::Object(String::from("land/fx/wnfslib/Config")),
                    &[JValue::from(cid), JValue::from(access_key)],
                );
            if config_res.is_ok() {
                let config_l = config_res
                    .ok()
                    .unwrap()
                    .l();
                if config_l.is_ok() {
                    let config = config_l
                        .ok()
                        .unwrap();
                    return config.into_inner();
                } else {
                    trace!("wnfsError occured in serialize_config config_l: {:?}", config_l.err().unwrap().to_string());
                    return JObject::null().into_inner();
                }
            } else {
                trace!("wnfsError occured in serialize_config config_res: {:?}", config_res.err().unwrap().to_string());
                return JObject::null().into_inner();
            }
        } else {
            trace!("wnfsError occured in serialize_config create_config_fn_res: {:?}", create_config_fn_res.err().unwrap().to_string());
            return JObject::null().into_inner();
        }
    }

    pub fn deserialize_cid(env: JNIEnv, jni_cid: JString) -> Cid {
        let cid: String = env.get_string(jni_cid).expect("Failed to parse cid").into();
        let cid = Cid::try_from(cid).unwrap();
        trace!("**********************deserialize_cid started**************");
        trace!(
            "**********************deserialize_cid cid={}",
            cid.to_string()
        );
        cid
    }

    pub fn serialize_cid(env: JNIEnv, cid: Cid) -> JString {
        trace!("**********************serialize_cid started**************");
        trace!(
            "**********************serialize_cid cid={:?}",
            cid.to_string()
        );
        let a: JString = env
            .new_string(cid.to_string())
            .expect("Failed to serialize cid")
            .into();
        a
    }

    pub fn serialize_access_key(env: JNIEnv, access_key: AccessKey) -> JString {
        env.new_string(serde_json::to_string(&access_key).unwrap())
            .expect("Failed to create private ref string")
            .into()
    }

    pub fn deserialize_access_key(env: JNIEnv, jni_access_key: JString) -> AccessKey {
        let access_key: String = env
            .get_string(jni_access_key)
            .expect("Failed to parse private ref")
            .into();
        let pref = serde_json::from_str::<AccessKey>(&access_key).unwrap();
        trace!("**********************deserialize_pref started**************");
        trace!("**********************deserialize_pref pref={:?}", pref);
        pref
    }

    pub fn prepare_path_segments(env: JNIEnv, jni_path_segments: JString) -> Vec<String> {
        let path: String = env
            .get_string(jni_path_segments)
            .expect("Failed to parse input path segments")
            .into();

        PrivateDirectoryHelper::parse_path(path)
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    pub fn prepare_ls_output(env: JNIEnv, ls_result: Vec<(String, Metadata)>) -> Result<Vec<u8>, String> {

        let mut result: Vec<u8> = Vec::new();

        let item_separator = "???".to_owned();
        let line_separator = "!!!".to_owned();
                    for item in ls_result.iter() {
                        
                        let created = item.1.clone().get_created();
                        let modification = item.1.clone().get_modified();
                        if created.is_some() && modification.is_some() {
                            let filename: String = item.0.clone().to_string().to_owned();
                            let creation_time: String = created.unwrap().to_string().to_owned();
                            let modification_time: String = modification.unwrap().to_string().to_owned();

                            let row_string: String = format!("{}{}{}{}{}{}", 
                                filename
                                , item_separator
                                , creation_time
                                , item_separator
                                , modification_time
                                , line_separator
                            );
                            let row_byte = row_string.as_bytes().to_vec();
                            result.append(&mut row_byte.to_owned());
                        }
                    }
                    Ok(result)

    }

    pub fn jbyte_array_to_vec(env: JNIEnv, jni_content: jbyteArray) -> Vec<u8> {
        env.convert_byte_array(jni_content)
            .expect("converting jbyteArray to Vec<u8>")
            .into()
    }

    pub fn vec_to_jbyte_array(env: JNIEnv, jni_content: Vec<u8>) -> jbyteArray {
        env.byte_array_from_slice(jni_content.as_slice())
            .expect("converting Vec<u8> to jbyteArray")
            .into()
    }
}

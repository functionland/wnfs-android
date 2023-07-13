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
    use wnfs::common::Metadata;
    use log::{trace, Level};
    extern crate android_logger;
    use android_logger::Config;
    use anyhow::Result;
    use wnfsutils::blockstore::{FFIFriendlyBlockStore, FFIStore};
    use wnfsutils::private_forest::PrivateDirectoryHelper;


    #[derive(Clone)]
    struct JNIStore<'a> {
        env: JNIEnv<'a>,
        fula_client: JObject<'a>,
    }

    impl<'a> JNIStore<'a> {
        fn  new(env: JNIEnv<'a>, fula_client: JObject<'a>) -> Self {
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

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_initRustLogger(_: JNIEnv, _: JClass) {
        android_logger::init_once(Config::default().with_min_level(Level::Trace));
    }



    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_loadWithWNFSKeyNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_wnfs_key: jbyteArray,
        jni_cid: JString,
    ) -> jobject  {
        trace!("**********************loadWithWNFSKeyNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = &mut FFIFriendlyBlockStore::new(Box::new(store));
        let wnfs_key: Vec<u8> = jbyte_array_to_vec(env, jni_wnfs_key);
        let forest_cid = deserialize_cid(env, jni_cid);
        let helper_res = PrivateDirectoryHelper::synced_load_with_wnfs_key(block_store, forest_cid, wnfs_key);
        trace!("**********************loadWithWNFSKeyNative finished**************");
        if helper_res.is_ok() {
            unsafe {
            serialize_result(env, None)
            }
        } else {
            let msg = helper_res.err().unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_loadWithWNFSKeyNative: {:?}", msg);
            unsafe {
                serialize_result(env, Some(msg))
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_initNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_wnfs_key: jbyteArray,
    ) -> jobject {
        trace!("**********************createRootDirNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = &mut FFIFriendlyBlockStore::new(Box::new(store));
        let wnfs_key: Vec<u8> = jbyte_array_to_vec(env, jni_wnfs_key);
        let helper_res = PrivateDirectoryHelper::synced_init(block_store, wnfs_key);
        
        if helper_res.is_ok() {
            let (_, _, cid) = helper_res.unwrap();
            unsafe {
            serialize_config_result(env, None, Some(cid))
            }
        } else {
            let msg = helper_res.err().unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_initNative: {:?}", msg.to_owned());
            unsafe {
                serialize_config_result(env, Some(msg.to_owned()), None)
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_writeFileFromPathNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        
        jni_path_segments: JString,
        jni_filename: JString,
    ) -> jobject {
        trace!("**********************writeFileFromPathNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = &mut FFIFriendlyBlockStore::new(Box::new(store));
        let cid = deserialize_cid(env, jni_cid);
        let helper_res = PrivateDirectoryHelper::synced_reload(block_store, cid);

        if helper_res.is_ok() {
            let helper =&mut helper_res.ok().unwrap();
                let path_segments = prepare_path_segments(env, jni_path_segments);

                let filename: String = env
                    .get_string(jni_filename)
                    .expect("Failed to parse input path segments")
                    .into();

                let write_file_result = 
                    helper.synced_write_file_from_path(&path_segments, &filename);
                    trace!("**********************writeFileFromPathNative finished**************");
                if write_file_result.is_ok() {
                    let cid = write_file_result.ok().unwrap();
                    unsafe{
                        return serialize_config_result(env, None, Some(cid));
                    }
                } else {
                    let msg = write_file_result.err().unwrap();
                    trace!("wnfsError in Java_land_fx_wnfslib_Fs_writeFileFromPathNative: {:?}", msg);
                    unsafe{
                        return serialize_config_result(env, Some(msg), None);
                    }
                }

        } else {
            let msg = &mut helper_res.err().unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_writeFileFromPathNative: {:?}", msg.to_owned());
            unsafe{
                return serialize_config_result(env, Some(msg.to_owned()), None);
            }
        }
        
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_readFilestreamToPathNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        
        jni_path_segments: JString,
        jni_filename: JString,
    ) -> jstring {
        trace!("wnfs11 **********************readFilestreamToPathNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = &mut FFIFriendlyBlockStore::new(Box::new(store));
        let cid = deserialize_cid(env, jni_cid);
        let helper_res = PrivateDirectoryHelper::synced_reload(block_store, cid);
        if helper_res.is_ok(){
            let helper =&mut helper_res.ok().unwrap();
            let path_segments = prepare_path_segments(env, jni_path_segments);
            let filename: String = env
                .get_string(jni_filename)
                .expect("Failed to parse input path segments")
                .into();
            trace!("wnfs11 **********************readFilestreamToPathNative filename created**************");
            let result = helper.synced_read_filestream_to_path(&filename, &path_segments, 0);
            trace!("wnfs11 **********************readFilestreamToPathNative finished**************");
            if result.is_ok() {
                unsafe{
                    return serialize_string_result(env, None, Some(filename));
                }
            } else {
                let err = result.err().unwrap();
                trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_readFilestreamToPathNative on result: {:?}", err.to_owned());
                unsafe{
                    return serialize_string_result(env,  Some(err.to_owned()), None);
                }
            }
        } else{
            let msg = helper_res.err().unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_readFilestreamToPathNative: {:?}", msg.to_owned());
            unsafe{
                return serialize_string_result(env, Some(msg.to_owned()), None);
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_readFileToPathNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_path_segments: JString,
        jni_filename: JString,
    ) -> jstring {
        trace!("wnfs11 **********************readFileToPathNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = &mut FFIFriendlyBlockStore::new(Box::new(store));
        let cid = deserialize_cid(env, jni_cid);
        let helper_res =PrivateDirectoryHelper::synced_reload(block_store, cid);

        if helper_res.is_ok() {
            let helper = &mut helper_res.ok().unwrap();
            let path_segments = prepare_path_segments(env, jni_path_segments);
            let filename: String = env
                .get_string(jni_filename)
                .expect("Failed to parse input path segments")
                .into();
            trace!("wnfs11 **********************readFileToPathNative filename created**************");
            let result = helper.synced_read_file_to_path(&path_segments, &filename);
            trace!("wnfs11 **********************readFileToPathNative finished**************");
            if result.is_ok() {
                unsafe{
                    return serialize_string_result(env, None, Some(filename));
                }
            } else {
                let err = result.err().unwrap();
                trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_readFileToPathNative on result: {:?}", err.to_owned());
                unsafe{
                    return serialize_string_result(env,  Some(err.to_owned()), None);
                }
            }
        } else{
            let msg = helper_res.err().unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_readFileToPathNative: {:?}", msg.to_owned());
            unsafe{
                return serialize_string_result(env, Some(msg.to_owned()), None);
            }
        }

    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_writeFileNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        
        jni_path_segments: JString,
        jni_content: jbyteArray,
    ) -> jobject {
        trace!("**********************writeFileNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = &mut FFIFriendlyBlockStore::new(Box::new(store));
        let cid = deserialize_cid(env, jni_cid);
        let helper_res =PrivateDirectoryHelper::synced_reload(block_store, cid);

        if helper_res.is_ok() {
            let helper = &mut helper_res.ok().unwrap();
            let path_segments = prepare_path_segments(env, jni_path_segments);
            let content = jbyte_array_to_vec(env, jni_content);

            let write_file_res = 
                helper.synced_write_file(&path_segments, content, 0);
            trace!("**********************writeFileNative finished**************");
            if write_file_res.is_ok() {
                let cid = write_file_res.ok().unwrap();
                unsafe{
                    let config: jobject = serialize_config_result(env, None, Some(cid));
                    return config;
                }
            } else {
                let msg = write_file_res
                    .err()
                    .unwrap();
                trace!("wnfsError in Java_land_fx_wnfslib_Fs_writeFileNative: {:?}", msg);
                unsafe{
                    return serialize_config_result(env,  Some(msg.to_owned()), None);
                }
            }
        } else{
            let msg = helper_res.err().unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_writeFileNative: {:?}", msg.to_owned());
            unsafe{
                return serialize_config_result(env, Some(msg.to_owned()), None);
            }
        }

        
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_readFileNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        
        jni_path_segments: JString,
    ) -> jbyteArray {
        trace!("**********************readFileNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = &mut FFIFriendlyBlockStore::new(Box::new(store));
        let cid = deserialize_cid(env, jni_cid);
        let helper_res =PrivateDirectoryHelper::synced_reload(block_store, cid);

        if helper_res.is_ok() {
            let helper = &mut helper_res.ok().unwrap();
            let path_segments = prepare_path_segments(env, jni_path_segments);
            trace!("**********************readFileNative finished**************");
            let result = helper.synced_read_file(&path_segments);
            if result.is_ok() {
                unsafe{
                    return serialize_bytes_result(env, None, Some(result.ok().unwrap()));
                }
            }else{
                let msg = result
                    .err()
                    .unwrap();
                trace!("wnfsError in Java_land_fx_wnfslib_Fs_readFileNative: {:?}", msg);
                unsafe{
                    return serialize_bytes_result(env,  Some(msg.to_owned()), None);
                }
            }

        } else{
            let msg = helper_res.err().unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_readFileNative: {:?}", msg.to_owned());
            unsafe{
                return serialize_bytes_result(env, Some(msg.to_owned()), None);
            }
        }

    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_mkdirNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        
        jni_path_segments: JString,
    ) -> jobject {
        trace!("**********************mkDirNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = &mut FFIFriendlyBlockStore::new(Box::new(store));
        let cid = deserialize_cid(env, jni_cid);
        let helper_res =PrivateDirectoryHelper::synced_reload(block_store, cid);

        if helper_res.is_ok() {
            let helper = &mut helper_res.ok().unwrap();
            let path_segments = prepare_path_segments(env, jni_path_segments);
            let mkdir_res = helper.synced_mkdir(&path_segments);
            if mkdir_res.is_ok() {
                let cid = mkdir_res.ok().unwrap();
                trace!("**********************mkDirNative finished**************");
                unsafe{return serialize_config_result(env, None, Some(cid))}
            } else {
                let msg = mkdir_res
                    .err()
                    .unwrap();
                trace!("wnfsError in Java_land_fx_wnfslib_Fs_mkdirNative: {:?}", msg.to_owned());
                unsafe{return serialize_config_result(env, Some(msg.to_owned()), None);}
            }
        } else{
            let msg = helper_res.err().unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_mkDirNative: {:?}", msg.to_owned());
            unsafe{
                return serialize_config_result(env, Some(msg.to_owned()), None);
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_mvNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        
        jni_source_path_segments: JString,
        jni_target_path_segments: JString,
    ) -> jobject {
        trace!("**********************mvNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = &mut FFIFriendlyBlockStore::new(Box::new(store));
        let cid = deserialize_cid(env, jni_cid);
        let helper_res =PrivateDirectoryHelper::synced_reload(block_store, cid);

        if helper_res.is_ok() {
            let helper = &mut helper_res.ok().unwrap();
            let source_path_segments = prepare_path_segments(env, jni_source_path_segments);
            let target_path_segments = prepare_path_segments(env, jni_target_path_segments);
            let result = helper.synced_mv(&source_path_segments, &target_path_segments);
            trace!("**********************mvNative finished**************");
            if result.is_ok() {
                let cid = result.ok().unwrap();
                unsafe{
                    return serialize_config_result(env, None, Some(cid));
                }
            }else {
                let msg = result.err().unwrap();
                trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_mvNative: {:?}", msg.to_owned());
                unsafe{
                    return serialize_config_result(env, Some(msg.to_owned()), None);
                }
            }
    
        } else{
            let msg = helper_res.err().unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_mvNative: {:?}", msg.to_owned());
            unsafe{
                return serialize_config_result(env, Some(msg.to_owned()), None);
            }
        }


        
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_cpNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        
        jni_source_path_segments: JString,
        jni_target_path_segments: JString,
    ) -> jobject {
        trace!("**********************cpNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = &mut FFIFriendlyBlockStore::new(Box::new(store));
        let cid = deserialize_cid(env, jni_cid);
        let helper_res =PrivateDirectoryHelper::synced_reload(block_store, cid);

        if helper_res.is_ok() {
            let helper = &mut helper_res.ok().unwrap();
            let source_path_segments = prepare_path_segments(env, jni_source_path_segments);
            let target_path_segments = prepare_path_segments(env, jni_target_path_segments);
            let result = helper.synced_cp(&source_path_segments, &target_path_segments);
            trace!("**********************cpNative finished**************");
            if result.is_ok() {
                let cid = result.ok().unwrap();
                unsafe{
                    return serialize_config_result(env, None, Some(cid));
                }
            }else {
                let msg = result.err().unwrap();
                trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_cpNative: {:?}", msg.to_owned());
                unsafe{
                    return serialize_config_result(env, Some(msg.to_owned()), None);
                }
            }
    
        } else{
            let msg = helper_res.err().unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_cpNative: {:?}", msg.to_owned());
            unsafe{
                return serialize_config_result(env, Some(msg.to_owned()), None);
            }
        }
        
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_rmNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        
        jni_path_segments: JString,
    ) -> jobject {
        trace!("**********************rmNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = &mut FFIFriendlyBlockStore::new(Box::new(store));
        let cid = deserialize_cid(env, jni_cid);
        let helper_res =PrivateDirectoryHelper::synced_reload(block_store, cid);
        
        if helper_res.is_ok() {
            let helper = &mut helper_res.ok().unwrap();
            let path_segments = prepare_path_segments(env, jni_path_segments);
            let rm_res = helper.synced_rm(&path_segments);
            if rm_res.is_ok() {
                let cid = rm_res.ok().unwrap();
                trace!("**********************rmNative finished**************");
                unsafe{return serialize_config_result(env, None, Some(cid))}
            } else {
                let msg = rm_res
                    .err()
                    .unwrap();
                trace!("wnfsError in Java_land_fx_wnfslib_Fs_rmNative: {:?}", msg.to_owned());
                unsafe{return serialize_config_result(env, Some(msg.to_owned()), None);}
            }
        } else{
            let msg = helper_res.err().unwrap();
            trace!("wnfsError in Java_land_fx_wnfslib_Fs_rmNative: {:?}", msg.to_owned());
            unsafe{
                return serialize_config_result(env, Some(msg.to_owned()), None);
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_Fs_lsNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        
        jni_path_segments: JString,
    ) -> jbyteArray {
        trace!("**********************lsNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = &mut FFIFriendlyBlockStore::new(Box::new(store));
        let cid = deserialize_cid(env, jni_cid);
        let helper_res =PrivateDirectoryHelper::synced_reload(block_store, cid);

        if helper_res.is_ok() {
            let helper = &mut helper_res.ok().unwrap();
                let path_segments = prepare_path_segments(env, jni_path_segments);
                let ls_res = helper.synced_ls_files(&path_segments);
                if ls_res.is_ok() {
                    let output =
                        prepare_ls_output(ls_res.ok().unwrap());
                    trace!("**********************lsNative finished**************");
                    if output.is_ok() {
                        let res = output.ok().unwrap();
                        unsafe { return serialize_bytes_result(env, None, Some(res));}
                    } else {
                        let msg = output.err().unwrap().to_string();
                        trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_lsNative output: {:?}", msg.to_owned());
                        unsafe { return serialize_bytes_result(env, Some(msg), None);}
                    }
                } else {
                    let msg = ls_res.err().unwrap();
                    trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_lsNative ls_res: {:?}", msg.to_owned());
                    unsafe{
                        return serialize_bytes_result(env, Some(msg.to_owned()), None);
                    }
                }
        } else {
            let msg = helper_res.err().unwrap();
            trace!("wnfsError occured in Java_land_fx_wnfslib_Fs_lsNative forest_res: {:?}", msg.to_owned());
            unsafe{
                return serialize_bytes_result(env, Some(msg.to_owned()), None);
            }
        }
    }

    pub unsafe fn serialize_result(env: JNIEnv, err: Option<String>) -> jobject {
        trace!("**********************serialize_result started**************");
        create_result_object(env, "Result".into(), err, JObject::null().into())
    }

    pub unsafe fn serialize_bytes_result(env: JNIEnv, err: Option<String>, bytes: Option<Vec<u8>>) -> jobject {
        trace!("**********************serialize_result started**************");
        let result = match bytes {
                Some(bytes) => vec_to_jbyte_array(env, bytes),
                None => JObject::null().into_inner(),
        };
        create_result_object(env, "BytesResult".into(), err, result.into())
    }

    pub unsafe fn serialize_string_result(env: JNIEnv, err: Option<String>, text: Option<String>) -> jobject {
        trace!("**********************serialize_string_result started**************");
        let result = match text {
                Some(text) => serialize_string(env, text),
                None => JObject::null().into(),
        };
        create_result_object(env, "StringResult".into(), err, result.into())
    }

    pub fn serialize_config(env: JNIEnv, cid: Cid) -> jobject {
        // Get the Config class
        let config_class = env.find_class("land/fx/wnfslib/Config").unwrap();
        
        // Convert the Cid to a string
        let cid_string = env.new_string(cid.to_string()).unwrap();
    
        // Create a new Config object
        let create_config_object_fn_res = env
            .get_static_method_id(
                config_class,
                "create",
                format!("(Ljava/lang/String;)Lland/fx/wnfslib/Config;"),
            );
        if create_config_object_fn_res.is_ok() {
            let create_config_object_fn = create_config_object_fn_res.ok().unwrap();
            let result_res = env
                .call_static_method_unchecked(
                    config_class,
                    create_config_object_fn,
                    JavaType::Object(format!("land/fx/wnfslib/result/Config")),
                    &[JValue::Object(*cid_string)],
                ).expect("Couldn't create new Config object");
        
                return result_res.l().unwrap().into_inner();
        } else {
            trace!("wnfsError occured in serialize_config create_config_object_fn_res: {:?}", create_config_object_fn_res.err().unwrap().to_string());
            return JObject::null().into_inner();
        }
    }
    
    
    pub unsafe fn serialize_config_result(env: JNIEnv, err: Option<String>, cid: Option<Cid>) -> jobject {
        trace!("**********************serialize_config_result started**************");
        /*let result = match cid {
                Some(cid) => serialize_cid(env, cid),
                None => JObject::null().into(),
        };*/
        let result: jobject = match cid {
            Some(cid) => serialize_config(env, cid),
            None => JObject::null().into_inner(),
        };
        create_result_object(env, "ConfigResult".into(), err, result.into())
    }

    pub fn create_result_object(env: JNIEnv, java_class_name: String, err: Option<String>, result: JObject) -> jobject {
        let result_cls = env.find_class("land/fx/wnfslib/result/ConfigResult").unwrap();
        trace!("**********************create_result_object result_cls set**************");
        let create_result_fn_res = env
            .get_static_method_id(
                result_cls,
                "create",
                format!("(Ljava/lang/String;Lland/fx/wnfslib/Config;)Lland/fx/wnfslib/result/{};", java_class_name),
            );
        if create_result_fn_res.is_ok() {
            let create_result_fn = create_result_fn_res.ok().unwrap();

            trace!("**********************create_result_object create_result_fn set**************");
            let err_java = match err {
                Some(err) => serialize_string(env, err),
                None => JObject::null().into(),
            };
            trace!("**********************create_result_object almost finished**************");
            trace!("Result object: {:?}", result);
            let result_res = env
                .call_static_method_unchecked(
                    result_cls,
                    create_result_fn,
                    JavaType::Object(format!("land/fx/wnfslib/result/{}", java_class_name)),
                    &[JValue::from(err_java), JValue::from(result)],
                );
            if result_res.is_ok() {
                let result_l = result_res
                    .ok()
                    .unwrap()
                    .l();
                if result_l.is_ok() {
                    let result = result_l
                        .ok()
                        .unwrap();
                    return result.into_inner();
                } else {
                    trace!("wnfsError occured in create_result_object result_l: {:?}", result_l.err().unwrap().to_string());
                    return JObject::null().into_inner();
                }
            } else {
                trace!("wnfsError occured in create_result_object result_res: {:?}", result_res.err().unwrap().to_string());
                return JObject::null().into_inner();
            }
        } else {
            trace!("wnfsError occured in create_result_object create_result_fn_res: {:?}", create_result_fn_res.err().unwrap().to_string());
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


    pub fn serialize_string(env: JNIEnv, text: String) -> JString {
        trace!("**********************serialize_string started**************");
        trace!(
            "**********************serialize_string text={:?}",
            text
        );
        let a: JString = env
            .new_string(text)
            .expect("Failed to serialize text")
            .into();
        a
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

    pub fn prepare_ls_output(ls_result: Vec<(String, Metadata)>) -> Result<Vec<u8>, String> {

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

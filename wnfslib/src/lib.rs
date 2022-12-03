// #[cfg(target_os = "android")]
// #[allow(non_snake_case)]
// #[allow(unused_imports)]

pub mod android {
    extern crate jni;

    use jni::objects::{JClass, JString, JValue, JObject, JByteBuffer};
    use jni::sys::jlong;
    use jni::sys::{jint, jobject, jstring, jboolean, jbyteArray};
    use jni::JNIEnv;
    use jni::signature::{JavaType, Primitive};
    use libipld::Cid;
    use log::{debug, trace, Level};
    use serde::Deserialize;
    use serde::__private::de::Content;
    use wnfs::{Id, Metadata, private};
    use wnfs::private::PrivateRef;
    use std::{fs::File, path::{Path}};
    extern crate android_logger;
    use android_logger::Config;
    use wnfs::public::PublicDirectory;
    use chrono::Utc;
    use kv::*;
    use anyhow::Result;
    use wnfsutils::private_forest::PrivateDirectoryHelper;
    use wnfsutils::blockstore::{FFIStore, FFIFriendlyBlockStore};


    struct JNIStore<'a>{
        env: JNIEnv<'a>,
        fula_client: JObject<'a>
    }

    impl<'a> JNIStore<'a> {
        fn new(env: JNIEnv<'a>, fula_client: JObject<'a>) -> Self{
            Self { env, fula_client }
        }
    }

    impl<'a> FFIStore<'a> for JNIStore<'a> {

        /// Retrieves an array of bytes from the block store with given CID.
        fn get_block(&self, cid: Vec<u8>) -> Result<Vec<u8>>{
            trace!("**********************get_block started**************");
            let get_fn = self.env
                .get_method_id(
                    self.fula_client,
                    "get",
                    "([B)[B",
                )
                .unwrap();
    
            let cidJByteArray = vec_to_jbyteArray(self.env, cid);
            let dataJByteArray = self.env
            .call_method_unchecked(
                self.fula_client,
                get_fn,
                JavaType::Object(String::from("[B")),
                &[
                    JValue::from(cidJByteArray),
                ],
            )
            .unwrap()
            .l()
            .unwrap();

            let data = jbyteArray_to_vec(self.env, dataJByteArray.into_inner());
            trace!("**********************get_block finished**************");
            Ok(data)
        }

        /// Stores an array of bytes in the block store.
        fn put_block(&self, bytes: Vec<u8>, codec: i64) -> Result<Vec<u8>>{
            trace!("**********************put_block started**************");
            let put_fn = self.env
                .get_method_id(
                    self.fula_client,
                    "put",
                    "([BJ)[B",
                )
                .unwrap();
            trace!("**********************put_block put_fn done**************");        
            let dataJByteArray = vec_to_jbyteArray(self.env, bytes);
            trace!("**********************put_block dataJByteArray done**************");
            let cidJByteArray = self.env
            .call_method_unchecked(
                self.fula_client,
                put_fn,
                JavaType::Object(String::from("[B")),
                &[
                    JValue::from(dataJByteArray),
                    JValue::from(codec),
                ],
            )
            .unwrap_or_else(|_err: jni::errors::Error| {
                trace!("**********************put_block first unwrap error**************");
                Err(_err).unwrap()
            })
            .l()
            .unwrap_or_else(|_err: jni::errors::Error| {
                trace!("**********************put_block second unwrap error**************");
                Err(_err).unwrap()
            });
            trace!("**********************put_block cidJByteArray done**************");
            let cid = jbyteArray_to_vec(self.env, cidJByteArray.into_inner());
            trace!("**********************put_block finished**************");
            Ok(cid)
        }
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_LibKt_initRustLogger(_: JNIEnv, _: JClass) {
        android_logger::init_once(Config::default().with_min_level(Level::Trace));
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_LibKt_createPrivateForestNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
    ) -> jstring {
        trace!("**********************createPrivateForest started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);
        trace!("**********************createPrivateForest finished**************");
        serialize_cid(env, helper.synced_create_private_forest().unwrap()).into_inner()
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_LibKt_createRootDirNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
    ) -> jobject {
        trace!("**********************createRootDirNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);
        let forest_cid = deserialize_cid(env, jni_cid);
        trace!("cid: {}", forest_cid);
        let forest = helper.synced_load_forest(forest_cid).unwrap();
        let (cid, private_ref) = helper.synced_init(forest);
        trace!("pref: {:?}", private_ref);
        trace!("**********************createRootDirNative finished**************");
        serialize_config(env, cid, private_ref)
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_LibKt_writeFileNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_private_ref: JString,
        jni_path_segments: JString,
        jni_content: jbyteArray,
    ) -> jobject {
        trace!("**********************writeFileNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);
        
        let cid = deserialize_cid(env, jni_cid);
        let private_ref = deserialize_private_ref(env, jni_private_ref);

        let forest = helper.synced_load_forest(cid).unwrap();
        let root_dir = helper.synced_get_root_dir(forest.to_owned(), private_ref).unwrap();
        let path_segments = prepare_path_segments(env, jni_path_segments);
        let content = jbyteArray_to_vec(env, jni_content);
        let (cid, private_ref) = helper.synced_write_file(forest.to_owned(), root_dir, &path_segments, content);
        trace!("**********************writeFileNative finished**************");
        serialize_config(env, cid, private_ref)
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_LibKt_readFileNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_private_ref: JString,
        jni_path_segments: JString,
    ) -> jbyteArray {
        trace!("**********************readFileNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);
        
        let cid = deserialize_cid(env, jni_cid);
        let private_ref = deserialize_private_ref(env, jni_private_ref);

        let forest = helper.synced_load_forest(cid).unwrap();
        let root_dir = helper.synced_get_root_dir(forest.to_owned(), private_ref).unwrap();
        let path_segments = prepare_path_segments(env, jni_path_segments);
        trace!("**********************readFileNative finished**************");
        vec_to_jbyteArray(env, helper.synced_read_file(forest.to_owned(), root_dir, &path_segments).unwrap())
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_LibKt_mkdirNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_private_ref: JString,
        jni_path_segments: JString,
    ) -> jstring {
        trace!("**********************mkDirNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);
        
        let cid = deserialize_cid(env, jni_cid);
        let private_ref = deserialize_private_ref(env, jni_private_ref);

        let forest = helper.synced_load_forest(cid).unwrap();
        let root_dir = helper.synced_get_root_dir(forest.to_owned(), private_ref).unwrap();
        let path_segments = prepare_path_segments(env, jni_path_segments);
        let (cid, private_ref) = helper.synced_mkdir(forest.to_owned(), root_dir, &path_segments);
        trace!("**********************mkDirNative finished**************");
        serialize_config(env, cid, private_ref)
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_LibKt_lsNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
        jni_private_ref: JString,
        jni_path_segments: JString,
    ) -> jstring {
        trace!("**********************lsNative started**************");
        let store = JNIStore::new(env, jni_fula_client);
        let block_store = FFIFriendlyBlockStore::new(Box::new(store));
        let helper = &mut PrivateDirectoryHelper::new(block_store);
        
        let cid = deserialize_cid(env, jni_cid);
        let private_ref = deserialize_private_ref(env, jni_private_ref);

        let forest = helper.synced_load_forest(cid).unwrap();
        let root_dir = helper.synced_get_root_dir(forest.to_owned(), private_ref).unwrap();
        let path_segments = prepare_path_segments(env, jni_path_segments);
        let output = prepare_ls_output(helper.synced_ls_files(forest.to_owned(), root_dir, &path_segments));
        trace!("**********************lsNative finished**************");
        env.new_string(output.join("\n")).
            expect("Failed to create new jstring").
            into_inner()
    }

    #[no_mangle]
    pub extern fn serialize_config(
        env: JNIEnv,
        cid: Cid,
        private_ref: PrivateRef,
    ) -> jobject {
        trace!("**********************serialize_config started**************");
        let config_cls = env.find_class("land/fx/wnfslib/Config").unwrap();
        //let  handler_class = reinterpret_cast<jclass>(env.new_global_ref(config_cls));
        trace!("**********************serialize_config config_cls set**************");
        let create_config_fn = env
            .get_static_method_id(
                config_cls,
                "create",
                "(Ljava/lang/String;Ljava/lang/String;)Lland/fx/wnfslib/Config;",
            )
            .unwrap();

        trace!("**********************serialize_config create_config_fn set**************");
        let cid = serialize_cid(env, cid);
        let private_ref = serialize_private_ref(env, private_ref);
        trace!("**********************serialize_config almost finished**************");
        let config = env
        .call_static_method_unchecked(
             config_cls,
            create_config_fn,
            JavaType::Object(String::from("land/fx/wnfslib/Config")),
            &[
                JValue::from(cid),
                JValue::from(private_ref),
            ],
        )
        .unwrap()
        .l()
        .unwrap();
        config.into_inner()
    }

    #[no_mangle]
    pub extern fn deserialize_cid(
        env: JNIEnv,
        jni_cid: JString,
    ) -> Cid {
        let cid: String = env.
            get_string(jni_cid).
            expect("Failed to parse cid").into();
        Cid::try_from(cid).unwrap()
    }

    #[no_mangle]
    pub extern fn serialize_cid(
        env: JNIEnv,
        cid: Cid,
    ) -> JString {
        env.new_string(cid.to_string()).
            expect("Failed to serialize cid").
            into()
    }

    #[no_mangle]
    pub extern fn serialize_private_ref(
        env: JNIEnv,
        private_ref: PrivateRef,
    ) -> JString {
        env.new_string(serde_json::to_string(&private_ref).unwrap()).
            expect("Failed to create private ref string").into()
    }

    #[no_mangle]
    pub extern fn deserialize_private_ref(
        env: JNIEnv,
        jni_private_ref: JString,
    ) -> PrivateRef {
        let private_ref: String = env.
            get_string(jni_private_ref).
            expect("Failed to parse private ref").
            into();
        serde_json::from_str::<PrivateRef>(&private_ref).unwrap()
    }


    #[no_mangle]
    pub extern fn prepare_path_segments(
        env: JNIEnv,
        jni_path_segments: JString,
    ) -> Vec<String> {
        let path: String = env
            .get_string(jni_path_segments)
            .expect("Failed to parse input path segments")
            .into();

        PrivateDirectoryHelper::parse_path(path).iter().
            map(|s| s.to_string()).
            collect()
    }

    #[no_mangle]
    pub extern fn prepare_ls_output(
        ls_result: Vec<(String, Metadata)>
    ) -> Vec<String> {
        ls_result.iter().
            map(|s| s.0.clone() ).
            collect()
    }

    #[no_mangle]
    pub extern fn jbyteArray_to_vec(
        env: JNIEnv,
        jni_content: jbyteArray,
    ) -> Vec<u8> {
        env.convert_byte_array(jni_content).
            expect("converting jbyteArray to Vec<u8>").
            into()
    }
    #[no_mangle]
    pub extern fn vec_to_jbyteArray(
        env: JNIEnv,
        jni_content: Vec<u8>,
    ) -> jbyteArray {
        env.byte_array_from_slice(jni_content.as_slice()).
            expect("converting Vec<u8> to jbyteArray").
            into()
    }
}

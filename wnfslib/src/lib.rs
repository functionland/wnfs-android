#[cfg(target_os = "android")]
#[allow(non_snake_case)]

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
    use wnfsutils::private_forest::PrivateDirectoryHelper;
    use wnfsutils::blockstore::FFIStore;


    struct JNIStore{
        env: JNIEnv,
        fula_client: JObject
    }
    impl JNIStore {
        fn new(env: JNIEnv, fula_client: JObject) -> Self{
            Self { env, fula_client }
        }
    }
    impl FFIStore for JNIStore {
        
        /// Retrieves an array of bytes from the block store with given CID.
        fn get_block(&self, cid: Vec<u8>) -> Result<Vec<u8>>{
            let get_fn = env
                .get_method_id(
                    self.fula_client,
                    "get",
                    "([B;)[B;",
                )
                .unwrap();
    
            let cidJByteArray = vec_to_jbyteArray(env, cid);
            let dataJByteArray = env
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

            let data = jbyteArray_to_vec(env, dataJByteArray);
            Ok(data)
        }

        /// Stores an array of bytes in the block store.
        fn put_block(&self, cid: Vec<u8>, bytes: Vec<u8>) -> Result<Vec<u8>>{
            let put_fn = env
                .get_method_id(
                    self.fula_client,
                    "put",
                    "([B;[B;);",
                )
                .unwrap();
    
            let cidJByteArray = vec_to_jbyteArray(env, cid);
            let dataJByteArray = vec_to_jbyteArray(env, bytes);
            env
            .call_method_unchecked(
                self.fula_client,
                get_fn,
                JavaType::Object(String::from("")),
                &[
                    JValue::from(cidJByteArray),
                    JValue::from(dataJByteArray),
                ],
            )
            .unwrap()
            .l()
            .unwrap();
            Ok(cidJByteArray)
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
        let store = JNIStore::new(env, jni_fula_client);
        let helper = &mut PrivateDirectoryHelper::new(store);
        trace!("**********************cp1**************");
        serialize_cid(env, helper.synced_new_private_forest()).into_inner()
    }

    #[no_mangle]
    pub extern "C" fn Java_land_fx_wnfslib_LibKt_createRootDirNative(
        env: JNIEnv,
        _: JClass,
        jni_fula_client: JObject,
        jni_cid: JString,
    ) -> jobject {
        let store = JNIStore::new(env, jni_fula_client);
        let helper = &mut PrivateDirectoryHelper::new(store);
        let forest_cid = deserialize_cid(env, jni_cid);
        trace!("cid: {}", forest_cid);
        let forest = helper.synced_load_forest(forest_cid);
        let (cid, private_ref) = helper.synced_make_root_dir(forest);
        trace!("pref: {:?}", private_ref);

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
        let store = JNIStore::new(env, jni_fula_client);
        let helper = &mut PrivateDirectoryHelper::new(store);

        let cid = deserialize_cid(env, jni_cid);
        let private_ref = deserialize_private_ref(env, jni_private_ref);

        let forest = helper.synced_load_forest(cid);
        let root_dir = helper.synced_get_root_dir(forest.to_owned(), private_ref);
        let path_segments = prepare_path_segments(env, jni_path_segments);
        let content = jbyteArray_to_vec(env, jni_content);
        let (cid, private_ref) = helper.synced_write_file(forest.to_owned(), root_dir, &path_segments, content);
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
        let store = JNIStore::new(env, jni_fula_client);
        let helper = &mut PrivateDirectoryHelper::new(store);

        let cid = deserialize_cid(env, jni_cid);
        let private_ref = deserialize_private_ref(env, jni_private_ref);

        let forest = helper.synced_load_forest(cid);
        let root_dir = helper.synced_get_root_dir(forest.to_owned(), private_ref);
        let path_segments = prepare_path_segments(env, jni_path_segments);
        vec_to_jbyteArray(env, helper.synced_read_file(forest.to_owned(), root_dir, &path_segments))
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
        let store = JNIStore::new(env, jni_fula_client);
        let helper = &mut PrivateDirectoryHelper::new(store);

        let cid = deserialize_cid(env, jni_cid);
        let private_ref = deserialize_private_ref(env, jni_private_ref);

        let forest = helper.synced_load_forest(cid);
        let root_dir = helper.synced_get_root_dir(forest.to_owned(), private_ref);
        let path_segments = prepare_path_segments(env, jni_path_segments);
        let (cid, private_ref) = helper.synced_mkdir(forest.to_owned(), root_dir, &path_segments);
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
        let store = JNIStore::new(env, jni_fula_client);
        let helper = &mut PrivateDirectoryHelper::new(store);

        let cid = deserialize_cid(env, jni_cid);
        let private_ref = deserialize_private_ref(env, jni_private_ref);

        let forest = helper.synced_load_forest(cid);
        let root_dir = helper.synced_get_root_dir(forest.to_owned(), private_ref);
        let path_segments = prepare_path_segments(env, jni_path_segments);
        let output = prepare_ls_output(helper.synced_ls_files(forest.to_owned(), root_dir, &path_segments));
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
        let config_cls = env.find_class("land/fx/wnfslib/Config").unwrap();

        let create_config_fn = env
            .get_static_method_id(
                config_cls,
                "create",
                "(Ljava/lang/String;Ljava/lang/String;)Lland/fx/wnfslib/Config;",
            )
            .unwrap();


        let cid = serialize_cid(env, cid);
        let private_ref = serialize_private_ref(env, private_ref);

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

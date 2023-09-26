#[cfg(test)]
mod android_tests {
    extern crate jni;
    use crate::android::*;
    use jni::{
        objects::{JClass, JObject, JString},
        signature::JavaType,
        sys::{jbyteArray, jobject, jsize, jstring},
        JNIEnv,
    };
    use jvm::jvm::{unwrap, Jvm};
    use std::fs::File;
    use std::io::Write;
    use std::{fs, ptr};
    use wnfs::common::CODEC_DAG_CBOR;
    use wnfsutils::{
        blockstore::{FFIFriendlyBlockStore, FFIStore},
        kvstore::KVBlockStore,
    };

    fn create_dummy_file(filename: &str, file_size: usize) -> std::io::Result<()> {
        let chunk = [0u8; 1024]; // 1 KB of zeros
        let mut file = File::create(filename)?;

        for _ in 0..(file_size / 1024) {
            file.write_all(&chunk)?;
        }

        Ok(())
    }

    fn generate_dummy_data(size: usize) -> Vec<u8> {
        vec![0u8; size]
    }

    fn get_string(env: JNIEnv, obj: jobject) -> String {
        let jni_string = JString::from(obj);
        println!("jni_string");
        let r_string: String = env
            .get_string(jni_string.into())
            .expect("Failed to parse cid")
            .into();
        return r_string;
    }

    fn java_byte_array_to_string(
        env: JNIEnv,
        java_byte_array: jobject,
    ) -> Result<String, jni::errors::Error> {
        let java_byte_array = unsafe { JObject::from(java_byte_array) };
        let byte_array = java_byte_array.into_inner() as jbyteArray;
        let len = env.get_array_length(byte_array)? as jsize;
        let mut buf: Vec<i8> = vec![0; len as usize];

        env.get_byte_array_region(byte_array, 0, &mut buf)?;

        let u8_buf: Vec<u8> = unsafe { std::mem::transmute(buf) };
        let result_str = String::from_utf8(u8_buf).ok().unwrap();

        Ok(result_str)
    }

    fn get_cid(env: JNIEnv, obj: jobject) -> JString {
        let out = env
            .call_method(obj, "getCid", "()Ljava/lang/String;", &[])
            .unwrap_or_else(|_err: jni::errors::Error| panic!("wnfsError test ERE1: {}", _err))
            .l()
            .unwrap_or_else(|_err: jni::errors::Error| panic!("wnfsError test HERE2: {}", _err));
        return JString::from(out);
    }

    fn get_result(env: JNIEnv, obj: JObject, retClass: String) -> jobject {
        let obj_class = env
            .get_object_class(obj)
            .expect("Couldn't get object class");

        let out = env
            .call_method(obj, "getResult", "()Ljava/lang/Object;", &[])
            .unwrap_or_else(|_err: jni::errors::Error| panic!("wnfsError test ERE1: {}", _err))
            .l()
            .unwrap_or_else(|_err: jni::errors::Error| panic!("wnfsError test HERE2: {}", _err));
        let config_result_class = env.find_class(retClass).expect("Class not found");
        if env
            .is_instance_of(obj, config_result_class)
            .expect("is_instance_of failed")
        {
            // obj is of type ConfigResult
            // You can now safely call methods that are specific to ConfigResult on obj
        }
        return out.into_inner();
    }

    fn convert_to_jstring<'a>(
        env: &'a JNIEnv,
        rust_string: String,
    ) -> jni::errors::Result<JString<'a>> {
        env.new_string(rust_string)
    }

    #[test]
    fn test_overall() {
        unsafe {
            let itteration = 15;
            let jvm = Jvm::new();

            let env = jvm.env;
            let class = env
                .find_class("land/fx/wnfslib/result/ConfigResult")
                .expect("Error on class");

            let empty_key: Vec<u8> = vec![0; 32];
            let jni_wnfs_key = vec_to_jbyte_array(env, empty_key);
            let jni_fula_client = unwrap(
                &env,
                env.new_object("land/fx/wnfslib/InMemoryDatastore", "()V", &[]),
            );
            let jclass: JClass<'_> = JObject::null().into();
            let config =
                Java_land_fx_wnfslib_Fs_initNative(env, jclass, jni_fula_client, jni_wnfs_key);

            let jni_config = get_result(env, config.into(), "land/fx/wnfslib/Config".into());
            let jni_cid = get_cid(env, jni_config.into());
            let cid_value = deserialize_cid(env, jni_cid.into());
            println!("cid_value: {:?}", cid_value);
            assert!(
                !cid_value.to_string().is_empty(),
                "Cid value should not be empty"
            );

            //mkdir
            println!("*******************Starting mkdir******************");
            let rust_path = "root".to_string();
            let java_path =
                convert_to_jstring(&env, rust_path).expect("Couldn't convert to JString");
            let config = Java_land_fx_wnfslib_Fs_mkdirNative(
                env,
                jclass,
                jni_fula_client,
                jni_cid.into(),
                java_path,
            );
            let jni_config = get_result(env, config.into(), "land/fx/wnfslib/Config".into());
            let mut jni_cid = get_cid(env, jni_config.into());
            let cid_value = deserialize_cid(env, jni_cid.into());
            println!("cid_value mkdir: {:?}", cid_value);
            assert!(
                !cid_value.to_string().is_empty(),
                "Cid value should not be empty"
            );

            //mkdir test in itteration
            let mkdir_itteration = 2;
            println!(
                "cid_value before {} mkdir: {:?}",
                mkdir_itteration, cid_value
            );
            for i in 1..=2 {
                // Loop 10 times
                println!("*******************Starting mkdir {}******************", i);

                // Create a unique directory name by appending the loop counter to the base name
                let rust_path = format!("root/test_{}", i);

                let java_path =
                    convert_to_jstring(&env, rust_path).expect("Couldn't convert to JString");

                let config = Java_land_fx_wnfslib_Fs_mkdirNative(
                    env,
                    jclass,
                    jni_fula_client,
                    jni_cid.into(),
                    java_path,
                );

                let jni_config = get_result(env, config.into(), "land/fx/wnfslib/Config".into());
                jni_cid = get_cid(env, jni_config.into());
                let cid_value = deserialize_cid(env, jni_cid.into());

                println!("cid_value mkdir {}: {:?}", i, cid_value);

                assert!(
                    !cid_value.to_string().is_empty(),
                    "Cid value should not be empty for mkdir {}",
                    i
                );
                let _ = env.delete_local_ref(jni_config.into());
                let _ = env.delete_local_ref(config.into());
            }
            let cid_value = deserialize_cid(env, jni_cid.into());
            println!(
                "cid_value after all {} mkdir: {:?} from jni_cid",
                mkdir_itteration, cid_value
            );

            //ls1
            println!("*******************Starting ls1******************");
            let filenames_initial = Java_land_fx_wnfslib_Fs_lsNative(
                env,
                jclass,
                jni_fula_client,
                jni_cid.into(),
                serialize_string(env, "root".into()),
            );
            let jni_filenames_initial = get_result(env, filenames_initial.into(), "[B".into());
            let filenames = java_byte_array_to_string(env, jni_filenames_initial)
                .ok()
                .unwrap();
            println!("filenames mkdir2: {:?}", filenames);
            assert!((1..=mkdir_itteration).all(|i| filenames.contains(&format!("test_{}", i))));

            //Small writes test in itteration
            let write_itteration = 15;
            println!(
                "cid_value before {} write: {:?}",
                write_itteration, cid_value
            );
            for i in 1..=write_itteration {
                println!("**************cid_value before test{}: {:?}", i, cid_value);
                println!(
                    "*******************Starting small write{}******************",
                    i
                );
                let mut file = File::create(format!("test{}.txt", i)).ok().unwrap();
                // Write content into the file
                let content = format!("Hello World {}", i);
                let _ = file.write_all(content.as_bytes());

                let config = Java_land_fx_wnfslib_Fs_writeFileStreamFromPathNative(
                    env,
                    jclass,
                    jni_fula_client,
                    jni_cid.into(),
                    serialize_string(env, format!("root/test.{}.txt", i).into()),
                    serialize_string(env, format!("test{}.txt", i).into()),
                );

                let jni_config = get_result(env, config.into(), "land/fx/wnfslib/Config".into());
                jni_cid = get_cid(env, jni_config.into());
                let cid_value = deserialize_cid(env, jni_cid.into());

                println!("**************cid_value after test{}: {:?}", i, cid_value);
                assert!(
                    !cid_value.to_string().is_empty(),
                    "Cid value should not be empty"
                );
                let _ = env.delete_local_ref(jni_config.into());
                let _ = env.delete_local_ref(config.into());
            }
            let cid_value = deserialize_cid(env, jni_cid.into());
            println!(
                "cid_value after all {} write: {:?} from jni_cid",
                write_itteration, cid_value
            );

            //ls1
            println!("*******************Starting ls2******************");
            let filenames_initial = Java_land_fx_wnfslib_Fs_lsNative(
                env,
                jclass,
                jni_fula_client,
                jni_cid.into(),
                serialize_string(env, "root".into()),
            );
            let jni_filenames_initial = get_result(env, filenames_initial.into(), "[B".into());
            let filenames = java_byte_array_to_string(env, jni_filenames_initial)
                .ok()
                .unwrap();
            println!("filenames wrtiefile2: {:?}", filenames);
            assert!((1..=write_itteration).all(|i| filenames.contains(&format!("test.{}.txt", i))));

            //write_file_stream large
            println!("*******************Starting write_file_stream******************");
            let file_size = 50 * 1024 * 1024; // 60 MB
            let _ = create_dummy_file("largefile_test.bin", file_size);
            let config = Java_land_fx_wnfslib_Fs_writeFileStreamFromPathNative(
                env,
                jclass,
                jni_fula_client,
                jni_cid.into(),
                serialize_string(env, "root/largeFileStream.bin".into()),
                serialize_string(env, "largefile_test.bin".into()),
            );
            let jni_config = get_result(env, config.into(), "land/fx/wnfslib/Config".into());
            let jni_cid = get_cid(env, jni_config.into());
            let cid_value = deserialize_cid(env, jni_cid.into());
            println!(
                "**************cid_value writeFileStreamLarge: {:?}",
                cid_value
            );
            assert!(
                !cid_value.to_string().is_empty(),
                "Cid value should not be empty"
            );

            //read_file_Stream
            println!("*******************Starting read_file_Stream******************");
            let _config = Java_land_fx_wnfslib_Fs_readFilestreamToPathNative(
                env,
                jclass,
                jni_fula_client,
                jni_cid.into(),
                serialize_string(env, "root/largeFileStream.bin".into()),
                serialize_string(env, "largefile_test_read.bin".into()),
            );
            let original_filesize = fs::metadata("largefile_test.bin").ok().unwrap().len();
            let read_filesize = fs::metadata("largefile_test_read.bin").ok().unwrap().len();
            println!(
                "original filsezie: {:?} vs read filesize: {:?}",
                original_filesize, read_filesize
            );
            assert_eq!(original_filesize, read_filesize, "File sizes do not match!");

            //ls2
            println!("*******************Starting ls2******************");
            let filenames_initial = Java_land_fx_wnfslib_Fs_lsNative(
                env,
                jclass,
                jni_fula_client,
                jni_cid.into(),
                serialize_string(env, "root".into()),
            );
            let jni_filenames_initial = get_result(env, filenames_initial.into(), "[B".into());
            let filenames = java_byte_array_to_string(env, jni_filenames_initial)
                .ok()
                .unwrap();
            println!("filenames writeFileStreamLarge: {:?}", filenames);
            assert!(filenames.contains("largeFileStream.bin"));

            //clean up
            let _ = fs::remove_file("largefile_test_read.bin");
            let _ = fs::remove_file("largefile_test.bin");
            let _ = fs::remove_file("test1.txt");
            let _ = fs::remove_file("test2.txt");
            let _ = fs::remove_file("test3.txt");

            // let names: Vec<u8> = filenames_initial.result.into();
            // println!("ls_initial. filenames_initial={:?}", names);
            // // Write file
            // let test_content = "Hello, World!";
            // fs::write("./tmp/test.txt", test_content.to_owned()).expect("Unable to write file");

            // // Read large file
            // {
            //     let large_data = vec![0u8; 500 * 1024 * 1024];
            //     cid = write_file_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/test_large.bin".to_string()),
            //         large_data.to_owned().into(),
            //     );

            //     let content_large = read_file_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/test_large.bin".to_string()),
            //     );

            //     let content: Vec<u8> = content_large.result.into();
            //     assert_eq!(content.to_owned(), large_data.to_owned());
            //     if true {
            //         return
            //     }
            // }
            // // Read file
            // {
            //     cid = write_file_from_path_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/testfrompath.txt".to_string()),
            //         RustString::from("./tmp/test.txt".to_string()),
            //     );

            //     let content_from_path = read_file_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/testfrompath.txt".to_string()),
            //     );

            //     let content = String::from_utf8(content_from_path.result.into()).unwrap();
            //     assert_eq!(content, test_content.to_owned().to_string());
            //     println!("read_file_from_path. content={}", content);
            // }
            // // Read content from path to path
            // {
            //     let content_from_path_topath = read_file_to_path_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/testfrompath.txt".to_string()),
            //         RustString::from("./tmp/test2.txt".to_string()),
            //     );
            //     let content_str: String = (content_from_path_topath).result.into();
            //     println!("content_from_path_topath={}", content_str);
            //     let read_content = fs::read_to_string(content_str).expect("Unable to read file");
            //     assert_eq!(read_content, test_content.to_string());
            //     println!("read_file_from_path_of_read_to. content={}", read_content);
            // }
            // // Read content from file stream to path
            // {
            //     let content_stream_from_path_topath = read_filestream_to_path_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/testfrompath.txt".to_string()),
            //         RustString::from("./tmp/teststream.txt".to_string()),
            //     );
            //     let content_str: String = content_stream_from_path_topath.result.into();
            //     println!("content_stream_from_path_topath={}", content_str);
            //     let read_content = fs::read_to_string(content_str).expect("Unable to read file");
            //     assert_eq!(read_content, test_content.to_string());
            //     println!("read_file_from_path_of_read_to. content={}", read_content);
            // }
            // // CP: target folder must exists
            // {
            //     let _len: usize = 0;
            //     let _capacity: usize = 0;

            //     cid = mkdir_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/test1".to_string()),
            //     );

            //     cid = cp_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/testfrompath.txt".to_string()),
            //         RustString::from("root/testfrompathcp.txt".to_string()),
            //     );

            //     let content_cp = read_file_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/testfrompathcp.txt".to_string()),
            //     );
            //     let content: String = String::from_utf8(content_cp.result.into()).unwrap();
            //     println!("cp. content_cp={}", content);
            //     assert_eq!(content, test_content.to_string());
            // }
            // // MV: target folder must exists
            // {
            //     let len: usize = 0;
            //     let capacity: usize = 0;
            //     cid = mv_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/testfrompath.txt".to_string()),
            //         RustString::from("root/testfrompathmv.txt".to_string()),
            //     );

            //     let content_mv = read_file_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/testfrompathmv.txt".to_string()),
            //     );
            //     println!("len: {}, cap: {}", len, capacity);
            //     let content: String = String::from_utf8(content_mv.result.into()).unwrap();
            //     println!("mv. content_mv={}", content);
            //     assert_eq!(content, test_content.to_string());
            // }
            // // RM#1
            // {
            //     let len: usize = 0;
            //     let capacity: usize = 0;
            //     cid = rm_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/testfrompathmv.txt".to_string()),
            //     );

            //     let content_rm1 = read_file_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/testfrompathmv.txt".to_string()),
            //     );
            //     println!("len: {}, cap: {}", len, capacity);
            //     let content: String = String::from_utf8(content_rm1.result.into()).unwrap();
            //     println!("rm#1. content_rm#1={}", content);
            //     assert_eq!(content, "".to_string());
            // }
            // // RM#2
            // {
            //     let len: usize = 0;
            //     let capacity: usize = 0;
            //     cid = rm_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/testfrompathcp.txt".to_string()),
            //     );

            //     let content_rm2 = read_file_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/testfrompathcp.txt".to_string()),
            //     );
            //     println!("len: {}, cap: {}", len, capacity);
            //     let content: String = String::from_utf8(content_rm2.result.into()).unwrap();
            //     println!("rm#1. content_rm#1={}", content);
            //     assert_eq!(content, "".to_string());
            // }
            // //
            // {
            //     println!(
            //         "********************** test content: {}",
            //         test_content.to_owned()
            //     );
            //     cid = write_file_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/test.txt".to_string()),
            //         test_content.as_bytes().to_vec().into(),
            //     );

            //     cid = mkdir_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/test1".to_string()),
            //     );

            //     let content_ls = ls_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root".to_string()),
            //     );

            //     let file_names = String::from_utf8(content_ls.result.into()).unwrap();
            //     println!("ls. fileNames={}", file_names);
            //     let content_test = read_file_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/test.txt".to_string()),
            //     );

            //     let content: String = String::from_utf8(content_test.result.into()).unwrap();
            //     println!("read. content={}", content);
            //     assert_eq!(content, test_content.to_string());
            // }
            // println!("All tests before reload passed");

            // // Testing reload Directory
            // {
            //     println!(
            //         "wnfs12 Testing reload with cid={} & wnfsKey={:?}",
            //         cid.to_string(),
            //         wnfs_key_string
            //     );
            //     load_with_wnfs_key_native(
            //         blockstore,
            //         wnfs_key_string.to_owned().into(),
            //         cid.into(),
            //     );

            //     let content_reloaded = read_file_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/test.txt".to_string()),
            //     );
            //     let content: String = String::from_utf8(content_reloaded.result.into()).unwrap();
            //     println!("read. content={}", content);
            //     assert_eq!(content, test_content.to_string());
            // }
            // // Read content from path to path (reloaded)
            // {
            //     let content_from_path_topath_reloaded = read_file_to_path_native(
            //         blockstore,
            //         cid.into(),
            //         RustString::from("root/test.txt".to_string()),
            //         RustString::from("./tmp/test2.txt".to_string()),
            //     );
            //     let content_str: String = content_from_path_topath_reloaded.result.into();
            //     println!("content_from_path_topath_reloaded={}", content_str);
            //     let read_content = fs::read_to_string(content_str).expect("Unable to read file");
            //     assert_eq!(read_content, test_content.to_string());
            //     println!("read_file_from_path_of_read_to. content={}", read_content);
            // }
        }
    }
}

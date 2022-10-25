#[cfg(target_os = "android")]
#[allow(non_snake_case)]
pub mod android {
    extern crate jni;

    use jni::objects::{JClass, JString, JValue, JObject};
    use jni::sys::jlong;
    use jni::sys::{jint, jobject, jstring, jboolean};
    use jni::JNIEnv;
    use log::{debug, trace, Level};
    use wnfs::Id;
    use std::{fs::File, path::{Path}};
    extern crate android_logger;
    use android_logger::Config;
    use wnfs::public::PublicDirectory;
    use chrono::Utc;

    #[no_mangle]
    pub extern "C" fn Java_com_functionland_lib_LibKt_initRustLogger(_: JNIEnv, _: JClass) {
        android_logger::init_once(Config::default().with_min_level(Level::Trace));
    }
    #[no_mangle]
    pub extern "C" fn Java_com_functionland_lib_LibKt_testWNFSNative(
        env: JNIEnv,
        _: JClass
    ) -> jstring {
        let dir = PublicDirectory::new(Utc::now());
        let dir_id = dir.get_id();

        trace!("id = {}", dir_id);

        env.new_string(dir_id)  
            .expect("Couldn't create java string!")
            .into_inner()
    }
}

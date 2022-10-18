#[cfg(target_os = "android")]
#[allow(non_snake_case)]
pub mod android {
    extern crate jni;

    use jni::objects::{JClass, JString, JValue, JObject};
    use jni::sys::jlong;
    use jni::sys::{jint, jobject, jstring, jboolean};
    use jni::JNIEnv;
    use log::{debug, trace, Level};
    use std::{fs::File, path::{Path}};
    extern crate android_logger;
    use android_logger::Config;
    use image::EncodableLayout;
    use url::Url;
    use jni::signature::{JavaType, Primitive};
    use wnfs::{PublicDirectory, Id};

    #[no_mangle]
    pub extern "C" fn Java_space_taran_wnfslib_LibKt_initRustLogger(_: JNIEnv, _: JClass) {
        android_logger::init_once(Config::default().with_min_level(Level::Trace));
    }
    #[no_mangle]
    pub extern "C" fn Java_com_functionland_app_testWNFSNative(
        env: JNIEnv,
        _: JClass
    ) -> jlong {
        let dir = PublicDirectory::new(Utc::now());
        trace!("id = {}", dir.get_id());

        dir.get_id().into()
    }
}

use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;

#[no_mangle]
pub extern "system" fn Java_com_caeles_host_CaelesBridge_nativeHealth(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    env.new_string("caeles-android-bridge-poc:ok")
        .expect("JNI string should be created")
        .into_raw()
}

#[no_mangle]
pub extern "system" fn Java_com_caeles_host_CaelesBridge_nativeList(
    mut env: JNIEnv,
    _class: JClass,
    _registry_path: JString,
) -> jstring {
    // PoC placeholder: next step is to call shared runtime APIs.
    env.new_string("[]")
        .expect("JNI string should be created")
        .into_raw()
}

#[no_mangle]
pub extern "system" fn Java_com_caeles_host_CaelesBridge_nativeRun(
    mut env: JNIEnv,
    _class: JClass,
    _manifest_path: JString,
) -> jstring {
    // PoC placeholder: next step is to call shared runtime APIs.
    env.new_string("{\"status\":\"todo\"}")
        .expect("JNI string should be created")
        .into_raw()
}

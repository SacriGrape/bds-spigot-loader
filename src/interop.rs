use std::sync::Mutex;
use jni::{InitArgsBuilder, JavaVM, JNIVersion};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref JNI_JVM: Mutex<JavaVM> = {
        let jvm_args = InitArgsBuilder::new()
            .version(JNIVersion::Invalid(655360))
            .option("-Xcheck:jni")
            .build()
            .unwrap();

        Mutex::new(JavaVM::new(jvm_args).expect("Failed to start JVM"))
    };
}

pub fn init() {
    println!("Testing JVM");
    println!("Testing JVM");
    let jvm = JNI_JVM.lock().expect("Failed to get JVM");
    println!("Testing JVM");
    let env = jvm.get_env().expect("Failed to get env");
    println!("Got both!");
}
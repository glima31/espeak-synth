fn main() {
    let data_dir =
        std::env::var("DEP_ESPEAK_SYS_DATA_DIR").expect("espeak-sys should export data-dir");

    println!("cargo:rustc-env=ESPEAK_NG_DATA_DIR={}", data_dir);
}

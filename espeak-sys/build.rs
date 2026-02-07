use cmake::Config;
use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let espeak_src = manifest_dir.join("espeak-ng");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // for faster builds
    unsafe {
        env::set_var(
            "CMAKE_BUILD_PARALLEL_LEVEL",
            std::thread::available_parallelism()
                .unwrap()
                .get()
                .to_string(),
        );
    }

    // disable what we don't need
    let mut cmake_conf = Config::new(&espeak_src);
    let dst = cmake_conf
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("ENABLE_TESTS", "OFF")
        .define("USE_ASYNC", "OFF")
        .define("USE_MBROLA", "OFF")
        .define("USE_LIBSONIC", "OFF")
        .define("USE_LIBPCAUDIO", "OFF")
        .define("USE_SPEECHPLAYER", "OFF")
        .define("COMPILE_INTONATIONS", "OFF")
        .define("COMPILE_PHONEMES", "OFF")
        .define("COMPILE_DICTIONARIES", "OFF")
        .always_configure(false)
        .very_verbose(env::var("CMAKE_VERBOSE").is_ok())
        .build();

    // Search paths
    println!("cargo:rustc-link-search={}", out_dir.join("lib").display());
    println!(
        "cargo:rustc-link-search={}",
        out_dir.join("build/src/ucd-tools").display()
    );

    if cfg!(windows) {
        println!(
            "cargo:rustc-link-search={}",
            out_dir.join("build/src/ucd-tools/Release").display()
        );
    }

    println!("cargo:rustc-link-search={}", dst.display());

    // Link
    println!("cargo:rustc-link-lib=static=espeak-ng");
    println!("cargo:rustc-link-lib=static=ucd");

    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=c++");
    }

    if cfg!(all(debug_assertions, windows)) {
        println!("cargo:rustc-link-lib=dylib=msvcrtd");
    }

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", espeak_src.display()))
        .clang_arg(format!(
            "-I{}",
            espeak_src.join("src").join("include").display()
        ))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Failed to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Failed to write bindings to file");

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=espeak-ng");
}

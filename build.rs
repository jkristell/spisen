use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {

    #[cfg(feature = "nucleo")]
    let memoryx = include_bytes!("memory_f401.x");
    #[cfg(feature = "feather")]
    let memoryx = include_bytes!("memory_f405.x");

    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(memoryx)
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory_f401.x`
    // here, we ensure the build script is only re-run when
    // `memory_f401.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");
}
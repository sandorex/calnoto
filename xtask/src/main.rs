use std::path::{Path, PathBuf};
use duct::cmd;

// TODO add android-debug android-release win32 for building on linux
const HELP: &str = r#"Command available for 'cargo xtask <cmd>':
  build [debug|release] - build for current platform (defaults to release)
  test [cxx|rust]       - runs rust and/or cxx tests
  clean                 - cleans all compilation leftovers
"#;

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

// TODO at this point just add clap cli
fn main() {
    let mut args = std::env::args();

    // print help when no arguments passed
    if args.len() <= 1 {
        eprintln!("{HELP}");
        std::process::exit(1);
    }

    // first arg is the path to this binary
    args.next();

    let cmd = args.next().unwrap();
    let arg = args.next().unwrap_or_default().to_lowercase();
    match cmd.to_lowercase().as_str() {
        "build" => match arg.as_str() {
            "debug" => build(true),
            // NOTE defaulting to release
            "release" | "" => build(false),
            "clean" => {
                println!("Cleaning up..");
                let _ = std::fs::remove_dir_all("target");
            }
            x @ _ => {
                eprintln!("Invalid build type {x:?}");
                std::process::exit(1);
            }
        },
        // "android" => unimplemented!("building for android is not yet implemented"),
        "test" => unimplemented!("testing is not yet implemented"),
        _ => {
            eprintln!("Invalid command {cmd:?}\n\n{HELP}");
        }
    }
}

const LIB_PREFIX: &str = if cfg!(target_family = "unix") {
    "lib"
} else {
    ""
};

const LIB_SUFFIX: &str = if cfg!(target_family = "unix") {
    ".a"
} else {
    ".lib"
};

const EXE_SUFFIX: &str = if cfg!(target_family = "windows") {
    ".exe"
} else {
    ""
};

fn build(debug: bool) {
    // always starting in root directory
    let root = project_root();
    std::env::set_current_dir(&root).unwrap();

    let build_type = if debug { "debug" } else { "release" };

    println!("Building rust core");

    // build the default package (core)
    (if debug {
        cmd!("cargo", "build")
    } else {
        cmd!("cargo", "build", "--release")
    })
        .run()
        .unwrap();

    let out_dir = root
        .join("target")
        .join(build_type);

    let cmake_out_dir = out_dir.join("cmake");

    let lib_path = out_dir.join(format!("{LIB_PREFIX}calnoto_core{LIB_SUFFIX}"));
    let lib_include = root.join("rust").join("include");

    println!("Configuring cmake");

    // Configure cmake only if it does not exist
    if !cmake_out_dir.exists() {
        cmd!("cmake", "-B", &cmake_out_dir,
            format!("-DRUST_LIB={}", lib_path.display()),
            format!("-DRUST_INCLUDE={}", lib_include.display()),
            if debug {
                "-DCMAKE_BUILD_TYPE=Debug"
            } else {
                "-DCMAKE_BUILD_TYPE=Release"
            })
            .run()
            .unwrap();
    }

    println!("Building final executable");

    cmd!("cmake", "--build", &cmake_out_dir)
        .run()
        .unwrap();

    println!("Executable compiled at: {}", cmake_out_dir.join(format!("calnoto{EXE_SUFFIX}")).display())
}

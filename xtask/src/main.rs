mod cli;

use std::path::{Path, PathBuf};
use duct::cmd;
use clap::Parser;

const BUILD_DIR: &str = "build";
const BUILD_DIR_ANDROID: &str = "build-android";

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn get_generator() -> String {
    if cfg!(target_os = "windows") {
        // let cmake pick the default for windows
        "".to_string()
    } else {
        match std::process::Command::new("ninja").output() {
            Ok(_) => "-GNinja".to_string(),
            // TODO is the space gonna be a problem?
            Err(_) => "-GUnix Makefiles".to_string(),
        }
    }
}

// TODO win32 build on linux
fn main() {
    let args = cli::Cli::parse();
    let root = project_root();

    match &args.cmd {
        cli::CliCommands::Build(x) => build(&root, &x),
        cli::CliCommands::BuildAndroid(x) => build_android(&root, &x),
        cli::CliCommands::Clean => {
            println!("Cleaning up build directories..");
            let _ = std::fs::remove_dir_all(root.join(BUILD_DIR));
            let _ = std::fs::remove_dir_all(root.join(BUILD_DIR_ANDROID));
        },
        _ => todo!(),
    }
}

fn build(root: &Path, args: &cli::CmdBuildArgs) {
    let build_dir = root.join(BUILD_DIR);

    println!(":: Building {}-{}", std::env::consts::OS, if args.debug { "debug" } else { "release" });
    if !build_dir.exists() {
        println!(":: Configuring cmake");
        cmd!("cmake",
            "-S", &root,
            "-B", &build_dir,
            get_generator(),
            if args.debug {
                "-DCMAKE_BUILD_TYPE=Debug"
            } else {
                "-DCMAKE_BUILD_TYPE=Release"
            })
            .run()
            .unwrap();
    }

    println!(":: Building using cmake");

    cmd!("cmake", "--build", &build_dir)
        .run()
        .unwrap();
}

fn build_android(root: &Path, args: &cli::CmdBuildArgs) {
    let build_dir = root.join(BUILD_DIR_ANDROID);

    // allow pointing to specific qt-cmake version
    let qt_cmake = std::env::var("QT_CMAKE").unwrap_or_else(|_| "qt-cmake".to_string());
    let sdk_root = std::env::var("ANDROID_SDK_ROOT").unwrap();
    let ndk_root = std::env::var("ANDROID_NDK_ROOT").unwrap();

    println!(":: Building android-{}", if args.debug { "debug" } else { "release" });
    if !build_dir.exists() {
        println!(":: Configuring cmake");
        cmd!(qt_cmake,
            format!("-DANDROID_SDK_ROOT={sdk_root}"),
            format!("-DANDROID_NDK_ROOT={ndk_root}"),
            "-S", &root,
            "-B", &build_dir,
            get_generator(),
            if args.debug {
                "-DCMAKE_BUILD_TYPE=Debug"
            } else {
                "-DCMAKE_BUILD_TYPE=Release"
            })
            .run()
            .unwrap();
    }

    println!(":: Building using cmake");

    cmd!("cmake", "--build", &build_dir)
        .run()
        .unwrap();
}


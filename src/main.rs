#[macro_use] extern crate clap;
#[macro_use] extern crate lazy_static;
extern crate directories;

use std::path::PathBuf;

fn main() {
    use std::process::{Command, Stdio, exit};

    let matches = {
        use clap::{App, Arg, SubCommand};
        App::new(crate_name!())
            .version(crate_version!())
            .author(crate_authors!(", "))
            .bin_name("cargo")
            .subcommand(SubCommand::with_name("rustc-ndk")
                .arg(Arg::with_name("target")
                    .long("target")
                    .value_name("TARGET")
                    .takes_value(true)
                    .required(true)
                    .help("The triple for the target (ex: armv7-linux-androideabi)"))
                .arg(Arg::with_name("platform")
                    .long("ndk-platform")
                    .value_name("PLATFORM")
                    .takes_value(true)
                    .required(true)
                    .help("The platform to target (example: 18)"))
                .arg(Arg::with_name("lib")
                    .long("lib")
                    .help("Build only this package's library"))
                .arg(Arg::with_name("bin")
                    .long("bin")
                    .value_name("BIN")
                    .takes_value(true)
                    .help("Build only the specified binary"))
                .arg(Arg::with_name("package")
                    .long("package")
                    .short("p")
                    .value_name("PACKAGE")
                    .takes_value(true)
                    .help("Package to build"))
                .arg(Arg::with_name("release")
                    .long("release")
                    .help("Build artifacts in release mode, with optimizations"))
                .arg(Arg::with_name("profile")
                    .long("profile")
                    .value_name("PROFILE")
                    .takes_value(true)
                    .help("Profile to build the selected target for"))
                .arg(Arg::with_name("features")
                    .long("features")
                    .value_name("FEATURES")
                    .takes_value(true)
                    .help("Space-separated list of features to activate"))
                .arg(Arg::with_name("all-features")
                    .long("all-features")
                    .help("Activate all available features"))
                .arg(Arg::with_name("no-default-features")
                    .long("no-default-features")
                    .help("Do not activate the `default` feature"))
                .arg(Arg::with_name("manifest-path")
                    .long("manifest-path")
                    .help("Path to Cargo.toml"))
                .arg(Arg::with_name("message-format")
                    .long("message-format")
                    .help("Error format [default: human]  [possible values: human, json]"))
                .arg(Arg::with_name("verbose")
                    .long("verbose")
                    .short("v")
                    .multiple(true)
                    .help("Use verbose output (-vv very verbose/build.rs output)"))
                .arg(Arg::with_name("frozen")
                    .long("frozen")
                    .help("Require Cargo.lock and cache are up to date"))
                .arg(Arg::with_name("lock")
                    .long("lock")
                    .help("Require Cargo.lock is up to date"))
                .arg(Arg::with_name("rustc-args")
                    .allow_hyphen_values(true)
                    .multiple(true)
                    .last(true))
            ).get_matches()
    };
    let matches = matches.subcommand_matches("rustc-ndk")
        .expect("rustc-ndk matches must be found");

    let target = matches.value_of("target").unwrap();
    let platform: u32 = matches.value_of("platform").unwrap()
        .parse().expect("platform should be an integer");

    let rustc_args: Vec<&str> = matches.values_of("rustc-args").map(|x| x.collect()).unwrap_or(vec![]);
    let cargo_args: Vec<&str> = {
        let mut args = vec!["--target", target];
        if matches.is_present("lib") {
            args.push("--lib");
        }
        if let Some(x) = matches.value_of("bin") {
            args.push("--bin");
            args.push(x);
        }
        if let Some(x) = matches.value_of("package") {
            args.push("--package");
            args.push(x);
        }
        if matches.is_present("release") {
            args.push("--release");
        }
        if let Some(x) = matches.value_of("profile") {
            args.push("--profile");
            args.push(x);
        }
        if let Some(x) = matches.value_of("features") {
            args.push("--features");
            args.push(x);
        }
        if matches.is_present("all-features") {
            args.push("--all-features");
        }
        if matches.is_present("no-default-features") {
            args.push("--no-default-features");
        }
        if let Some(x) = matches.value_of("manifest-path") {
            args.push("--manifest-path");
            args.push(x);
        }
        if let Some(x) = matches.value_of("message-format") {
            args.push("--message-format");
            args.push(x);
        }
        match matches.occurrences_of("verbose") {
            0 => (),
            1 => args.push("-v"),
            _ => args.push("-vv"),
        }
        if matches.is_present("frozen") {
            args.push("--frozen");
        }
        if matches.is_present("lock") {
            args.push("--lock");
        }
        args
    };

    //eprintln!("cargo_args: {:?}", &cargo_args);
    //eprintln!("rustc_args: {:?}", &rustc_args);

    let project_dirs = directories::ProjectDirs::from("com", "rayark", "davll.cargo-rustc-ndk");
    let cache_dir = project_dirs.cache_dir();

    let toolchain_path = cache_dir.join("ndk-standalone")
        .join(format!("android-{platform}.{target}", target = target, platform = platform));
    
    if toolchain_path.is_dir() == false {
        eprintln!("toolchain.android-{platform}.{target} does not exist, generating...", target = target, platform = platform);
        eprintln!("toolchain path = {:?}", &toolchain_path);

        let make_toolchain = ANDROID_NDK_ROOT.join("build/tools/make_standalone_toolchain.py");
        eprintln!("running {:?} ...", &make_toolchain);

        let status = Command::new("python")
            .arg(make_toolchain)
            .arg("--arch").arg(target_to_arch(target))
            .arg("--api").arg(format!("{}", platform))
            .arg("--install-dir").arg(&toolchain_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .expect("Success");

        match status.code().expect("exited with signal") {
            0 => (),
            x => exit(x),
        }

        assert!(toolchain_path.is_dir());
        eprintln!("toolchain.android-{platform}.{target} is generated!", target = target, platform = platform);
    } else {
        eprintln!("toolchain.android-{platform}.{target} exists", target = target, platform = platform);
        eprintln!("toolchain path = {:?}", &toolchain_path);
    }

    let clang_path = toolchain_path.join("bin/clang");
    let gcc_path = toolchain_path.join("bin").join(format!("{}gcc", target_gcc_prefix(target)));
    let gxx_path = toolchain_path.join("bin").join(format!("{}g++", target_gcc_prefix(target)));
    let ar_path = toolchain_path.join("bin").join(format!("{}ar", target_gcc_prefix(target)));
    let cflags = target_cflags(target);

    let status = Command::new(&*CARGO)
        .env(cc_env_target_cfg(target, "CC"), &gcc_path)
        .env(cc_env_target_cfg(target, "CXX"), &gxx_path)
        .env(cc_env_target_cfg(target, "AR"), &ar_path)
        .env(cc_env_target_cfg(target, "CFLAGS"),
            &combine_vec_str(cflags.into_iter(), ' '))
        .arg("rustc")
        .args(&cargo_args)
        .arg("--")
        .args(&rustc_args)
        .arg("-C").arg(format!("linker={}", clang_path.to_str().unwrap()))
        .arg("-C").arg(format!("ar={}", ar_path.to_str().unwrap()))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Success");

    match status.code().unwrap() {
        0 => (),
        x => exit(x),
    }
}

//fn cargo_env_target_cfg(target: &str, key: &str) -> String {
//    format!("CARGO_TARGET_{}_{}", &target.replace("-", "_"), key).to_uppercase()
//}

fn target_to_arch(target: &str) -> &str {
    match target {
        "aarch64-linux-android" => "arm64",
        "armv7-linux-androideabi" => "arm",
        "i686-linux-android" => "x86",
        "x86_64-linux-android" => "x86_64",
        _ => unimplemented!("Unknown target"),
    }
}

fn target_cflags(target: &str) -> Vec<&str> {
    match target {
        "aarch64-linux-android" =>
            vec!["-fPIE", "-fPIC"],
        "armv7-linux-androideabi" =>
            vec!["-march=armv7-a", "-mfloat-abi=softfp", "-mfpu=vfpv3-d16", "-fPIE", "-fPIC"],
        "i686-linux-android" =>
            vec!["-march=i686", "-mtune=intel", "-mssse3", "-mfpmath=sse", "-m32", "-fPIE", "-fPIC"],
        "x86_64-linux-android" =>
            vec!["-march=x86-64", "-msse4.2", "-mpopcnt", "-m64", "-mtune=intel", "-fPIE", "-fPIC"],
        _ => unimplemented!("unknown target"),
    }
}

fn target_gcc_prefix(target: &str) -> &str {
    match target {
        "aarch64-linux-android" => "aarch64-linux-android-",
        "armv7-linux-androideabi" => "arm-linux-androideabi-",
        "i686-linux-android" => "i686-linux-android-",
        "x86_64-linux-android" => "x86_64-linux-android-",
        _ => unimplemented!("unknown target"),
    }
}

fn cc_env_target_cfg(target: &str, key: &str) -> String {
    format!("{}_{}", key, target)
}

fn combine_vec_str<'a, I>(iter: I, sep: char) -> String
    where I: IntoIterator<Item = &'a str> + 'a,
          I::IntoIter: 'a,
{
    let mut iter = iter.into_iter();
    match iter.next() {
        Some(start) => {
            iter.fold(String::from(start), |mut buf, s| {
                buf.push(sep);
                buf.push_str(s);
                buf
            })
        },
        None => format!(""),
    }
}

lazy_static! {
    static ref CARGO: PathBuf = {
        use std::env;
        let s = env::var_os("CARGO")
            .expect("the environment variable CARGO must be set");
        PathBuf::from(s)
    };
    static ref ANDROID_SDK_ROOT: PathBuf = {
        use std::env;
        let s = env::var_os("ANDROID_SDK_ROOT")
            .or_else(|| env::var_os("ANDROID_HOME"))
            .expect("ANDROID_SDK_ROOT or ANDROID_HOME must be set to find Android SDK");
        PathBuf::from(s)
    };
    static ref ANDROID_NDK_ROOT: PathBuf = {
        use std::env;
        env::var_os("ANDROID_NDK_ROOT").map(PathBuf::from)
            .or_else(|| Some(ANDROID_SDK_ROOT.join("ndk-bundle")))
            .expect("ANDROID_NDK_ROOT must be set to find Android NDK")
    };
}

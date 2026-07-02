use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let profile = env::var("PROFILE").unwrap_or_default();
    let is_release = profile != "debug";

    slint_build::compile("ui/app-window.slint").expect("Slint build failed");

    match target_os.as_str() {
        "windows" => {
            let manifest_path = format!("windows/{}.exe.manifest", env!("CARGO_PKG_NAME"));
            println!("cargo::rerun-if-changed={manifest_path}");
            if std::path::Path::new(&manifest_path).exists() && is_release {
                println!(
                    "cargo::rustc-link-arg-bin={}=/MANIFEST:EMBED",
                    env!("CARGO_PKG_NAME")
                );
            }
        }
        "macos" => {
            println!("cargo::rustc-env=MACOSX_DEPLOYMENT_TARGET=10.15");
        }
        "linux" => {
            println!("cargo::rustc-env=SLINT_STYLE=fluent");
        }
        _ => {}
    }

    println!("cargo::rerun-if-changed=ui/");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");
}

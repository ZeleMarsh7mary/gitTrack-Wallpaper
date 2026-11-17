//! Embeds the application icon and the DPI-aware manifest into the daemon exe.
//! Requires `rc.exe` from the Windows SDK (installed with the MSVC build tools).
//!
//! The `.rc` is generated into `OUT_DIR` with absolute, forward-slash paths:
//! `embed-resource` runs `rc.exe` from the resource file's own directory, and
//! rc.exe cannot resolve a `..` component inside a resource path.

use std::path::{Path, PathBuf};

fn main() {
    let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    // `crates/<crate>` -> repository root.
    let repo_root = crate_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| crate_dir.clone());
    let assets = repo_root.join("assets");
    let icon = assets.join("icon.ico");
    let manifest = assets.join("app.manifest");
    println!("cargo:rerun-if-changed={}", icon.display());
    println!("cargo:rerun-if-changed={}", manifest.display());

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let rc = out_dir.join("git-track-wallpaper.rc");
    let icon_path = slash(&icon);
    let manifest_path = slash(&manifest);
    std::fs::write(
        &rc,
        format!("1 ICON \"{icon_path}\"\n1 24 \"{manifest_path}\"\n"),
    )
    .expect("write generated resource script");

    // The manifest carries DPI awareness; treat a missing rc.exe as a hard warning
    // so the loss of per-monitor DPI is visible instead of silent.
    if let Err(e) = embed_resource::compile(&rc, embed_resource::NONE).manifest_required() {
        println!("cargo:warning=embed_resource: {e}");
    }
}

fn slash(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

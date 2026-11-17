//! Embeds the application icon and the DPI-aware manifest into the .exe.
//! Requires `rc.exe` from the Windows SDK (installed with the MSVC build tools).

fn main() {
    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/app.manifest");
    println!("cargo:rerun-if-changed=assets/icon.ico");
    // The manifest carries DPI awareness; treat a missing rc.exe as a hard warning
    // so the loss of per-monitor DPI is visible instead of silent.
    if let Err(e) = embed_resource::compile("assets/app.rc", embed_resource::NONE).manifest_required()
    {
        println!("cargo:warning=embed_resource: {e}");
    }
}

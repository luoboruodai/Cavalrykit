fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=packaging/windows.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winres::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .set("ProductName", "CavalryKit")
            .set("FileDescription", "CavalryKit Script and Plugin Installer")
            .set("OriginalFilename", "CavalryKit.exe")
            .set_manifest_file("packaging/windows.manifest")
            .compile()
            .expect("Embedding the Windows icon/version/manifest is required");
    }
}

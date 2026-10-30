fn main() {
    // Only build Windows-specific resources on Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        // Link the .def file for DLL exports
        let def_path = std::path::Path::new("afrim-desktop-tsf.def")
            .canonicalize()
            .expect("afrim-desktop-tsf.def not found");
        println!("cargo:rustc-cdylib-link-arg=/DEF:{}", def_path.display());

        // Compile .rc resource file (version info, icon)
        if std::path::Path::new("res/afrim-desktop.rc").exists() {
            let _ = embed_resource::compile("res/afrim-desktop.rc", embed_resource::NONE);
        }
    }

    println!("cargo:rerun-if-changed=afrim-desktop-tsf.def");
    println!("cargo:rerun-if-changed=res/afrim-desktop.rc");
    println!("cargo:rerun-if-changed=build.rs");
}

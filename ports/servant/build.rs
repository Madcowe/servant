fn main() {
    #[cfg(target_os = "windows")]
    {
        use std::env;
        use std::fs;
        use std::path::Path;

        let out_dir = env::var("OUT_DIR").unwrap();
        let out_path = Path::new(&out_dir);
        
        // Find the target/<profile>/build directory
        let mut build_dir = out_path;
        let mut found_build = false;
        while let Some(parent) = build_dir.parent() {
            if parent.file_name().map(|n| n == "build").unwrap_or(false) {
                build_dir = parent;
                found_build = true;
                break;
            }
            build_dir = parent;
        }

        let mut found = false;
        if found_build {
            if let Ok(entries) = fs::read_dir(build_dir) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            if name.starts_with("mozangle-") {
                                let egl = path.join("out").join("libEGL.dll");
                                let gles = path.join("out").join("libGLESv2.dll");
                                if egl.exists() && gles.exists() {
                                    fs::copy(&egl, out_path.join("libEGL.dll")).unwrap();
                                    fs::copy(&gles, out_path.join("libGLESv2.dll")).unwrap();
                                    found = true;
                                    println!("cargo:warning=Successfully bundled EGL/GLES DLLs from {}", path.display());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        if !found {
            println!("cargo:warning=EGL or GLESv2 DLLs could not be found under target build directory. Bundling failed.");
        }
    }
}

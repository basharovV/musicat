fn main() {
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=framework=MediaPlayer");
    // #[cfg(target_os = "android")]
    // println!("cargo:rustc-link-lib=c++_shared");
    tauri_build::build()
}

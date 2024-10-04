fn main() {
    println!("cargo:rustc-link-lib=framework=MediaPlayer");
    tauri_build::build()
}

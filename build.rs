use std::env;
use std::path::PathBuf;
use std::process::Command;

const RNNOISE_URL: &str =
    "https://github.com/xiph/rnnoise/releases/download/v0.2/rnnoise-0.2.tar.gz";

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let rnnoise_dir = out_dir.join("rnnoise-0.2");

    if !rnnoise_dir.exists() {
        let resp = ureq::get(RNNOISE_URL).call().expect("failed to download");
        let reader = resp.into_body().into_reader();
        let decoder = flate2::read::GzDecoder::new(reader);

        let mut archive = tar::Archive::new(decoder);
        archive.unpack(&out_dir).expect("failed to unpack");
    }

    Command::new("./configure")
        .arg(format!("--prefix={}", out_dir.display()))
        .arg("--enable-x86-rtcd")
        .arg("--enable-static")
        .current_dir(&rnnoise_dir)
        .status()
        .unwrap();

    Command::new("make")
        .current_dir(&rnnoise_dir)
        .status()
        .unwrap();

    Command::new("make")
        .arg("install")
        .current_dir(&rnnoise_dir)
        .status()
        .unwrap();

    println!("cargo:rustc-link-lib=static=rnnoise");
    println!("cargo:rustc-link-search=native={}/lib", out_dir.display());

    let bindings = bindgen::Builder::default()
        .header(format!("{}/include/rnnoise.h", out_dir.display()))
        .generate()
        .expect("Unable to generate bindings");

    let bindings_path = out_dir.join("bindings.rs");
    bindings
        .write_to_file(&bindings_path)
        .expect("Couldn't write bindings");
}

//! Every ethos file generates the Rust module committed beside the
//! hand-written source; a stale module fails the build.

use ethos_zero::{Actualizing, File, Generating, Potential};

fn main() {
    let root = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("manifest"));
    println!("cargo:rerun-if-changed=ethos");
    let mut sources: Vec<_> = std::fs::read_dir(root.join("ethos"))
        .expect("ethos directory")
        .map(|entry| entry.expect("ethos entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "ethos")
        })
        .collect();
    sources.sort();
    for source_path in sources {
        let stem = source_path
            .file_stem()
            .expect("stem")
            .to_string_lossy()
            .into_owned();
        let generated_path = root.join("src/generated").join(format!("{stem}.rs"));
        println!("cargo:rerun-if-changed={}", generated_path.display());
        let source = std::fs::read_to_string(&source_path).expect("ethos source");
        let file = Potential::<File>::from(source)
            .actualize()
            .unwrap_or_else(|_| panic!("ethos/{stem}.ethos is not read as ethos"));
        let generated = file
            .generate()
            .unwrap_or_else(|_| panic!("ethos/{stem}.ethos does not generate"));
        let committed = std::fs::read_to_string(&generated_path).unwrap_or_default();
        assert!(
            generated == committed,
            "src/generated/{stem}.rs is not what ethos/{stem}.ethos generates"
        );
    }
}

use std::path::Path;
use std::process::Command;

fn main() {
    let frontend_path =
        Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../frontend");

    assert_eq!(
        Command::new("yarn")
            .current_dir(&frontend_path)
            .status()
            .unwrap()
            .code()
            .unwrap(),
        0,
    );
    assert_eq!(
        Command::new("yarn")
            .arg("build")
            .current_dir(&frontend_path)
            .status()
            .unwrap()
            .code()
            .unwrap(),
        0,
    );

    // rerun in any directory besides dist has changed
    for ref name in std::fs::read_dir(frontend_path)
        .unwrap()
        .filter_map(|f| f.ok())
        .filter(|f| f.file_name() != "dist" && f.file_name() != "node_modules")
        .map(|f| f.path().to_str().unwrap().to_string())
    {
        //println!("cargo:warning=dicks {}", name);
        println!("cargo:rerun-if-changed={}", name);
    }
}

fn main() {
    println!("cargo:rerun-if-changed=src/resources.xml");
    println!("cargo:rerun-if-changed=icons/");

    let status = std::process::Command::new("glib-compile-resources")
        .args(&[
            "--target=src/resources.gresource",
            "--sourcedir=icons",
            "src/resources.xml",
        ])
        .status()
        .expect("Failed to execute glib-compile-resources");

    if !status.success() {
        panic!("glib-compile-resources failed with status: {}", status);
    }
}

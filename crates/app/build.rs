fn main() {
    println!("cargo:rerun-if-changed=../../assets/branding/orbit-axion.ico");
    winresource::WindowsResource::new()
        .set_icon("../../assets/branding/orbit-axion.ico")
        .set("ProductName", "Orbit Axion")
        .set("FileDescription", "Orbit Axion")
        .compile()
        .expect("compile Windows application icon");
    slint_build::compile("../../ui/lab.slint").expect("compile Slint laboratory UI");
}

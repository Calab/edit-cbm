//! Embed the program icon (and version details) in the Windows .exe,
//! once there is an icon.

fn main() {
    println!("cargo:rerun-if-changed=../../assets/icons/edit-cbm.ico");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        if std::path::Path::new("../../assets/icons/edit-cbm.ico").exists() {
            res.set_icon("../../assets/icons/edit-cbm.ico");
        }
        res.set("ProductName", "Edit CBM");
        res.set("FileDescription", "Edit CBM");
        if let Err(e) = res.compile() {
            panic!("embedding the Windows details failed: {e}");
        }
    }
}

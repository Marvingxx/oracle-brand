fn main() {
    println!("cargo:rerun-if-changed=assets/mockup-studio.ico");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/mockup-studio.ico");
        res.set("ProductName", "Oracle Mockup Studio");
        res.set("FileDescription", "Oracle Mockup Studio");
        res.set("CompanyName", "Oracle");
        if let Err(e) = res.compile() {
            println!("cargo:warning=icon not embedded: {e}");
        }
    }
}

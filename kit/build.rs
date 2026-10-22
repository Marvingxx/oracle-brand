fn main() {
    println!("cargo:rerun-if-changed=assets/brand-kit.ico");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/brand-kit.ico");
        res.set("ProductName", "Oracle Brand Kit");
        res.set("FileDescription", "Oracle Brand Kit");
        res.set("CompanyName", "Oracle");
        if let Err(e) = res.compile() {
            println!("cargo:warning=icon not embedded: {e}");
        }
    }
}

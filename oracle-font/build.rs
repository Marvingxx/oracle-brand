fn main() {
    println!("cargo:rerun-if-changed=assets/oracle-font.ico");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/oracle-font.ico");
        res.set("ProductName", "Oracle Font");
        res.set("FileDescription", "Oracle Font");
        res.set("CompanyName", "Oracle");
        if let Err(e) = res.compile() {
            println!("cargo:warning=icon not embedded: {e}");
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=assets/oracle-assets.ico");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/oracle-assets.ico");
        res.set("ProductName", "Oracle Assets");
        res.set("FileDescription", "Oracle Assets");
        res.set("CompanyName", "Oracle");
        if let Err(e) = res.compile() {
            println!("cargo:warning=icon not embedded: {e}");
        }
    }
}

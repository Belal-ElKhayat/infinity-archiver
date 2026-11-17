use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.png");
    println!("cargo:rerun-if-changed=assets/icon.ico");

    #[cfg(windows)]
    {
        let ico_path = Path::new("assets/icon.ico");
        let png_path = Path::new("assets/icon.png");

        if png_path.exists() {
            if let Ok(img) = image::open(png_path) {
                let resized = img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
                resized.save_with_format(ico_path, image::ImageFormat::Ico).expect("Failed to save icon.ico");
            }
        }

        if ico_path.exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon("assets/icon.ico");
            res.set("ProductName", "Infinity Archiver");
            res.set("FileDescription", "Infinity Archiver - High-Speed Rust File Archiver");
            res.compile().expect("Failed to compile Windows resources");
        }
    }
}

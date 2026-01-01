use std::env;
use std::path::Path;

fn main() {
    // 只在Windows平台处理图标
    #[cfg(target_os = "windows")]
    {
        let out_dir = env::var("OUT_DIR").unwrap();

        // 检查图标文件是否存在
        if Path::new("assets/icon.png").exists() {
            println!("cargo:warning=Found icon file: assets/icon.png");

            // 尝试使用winres库来嵌入图标
            if let Err(e) = embed_icon() {
                println!("cargo:warning=Failed to embed icon: {}", e);
            }
        } else {
            println!("cargo:warning=Icon file not found: assets/icon.png");
        }
    }

    // 告诉cargo在这些文件改变时重新构建
    println!("cargo:rerun-if-changed=assets/icon.png");
    println!("cargo:rerun-if-changed=assets/icon.ico");
}

#[cfg(target_os = "windows")]
fn embed_icon() -> Result<(), Box<dyn std::error::Error>> {
    // 首先尝试转换PNG到ICO
    convert_png_to_ico()?;

    // 然后嵌入ICO文件
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.set_language(0x0409); // 英语
    res.compile()?;

    println!("cargo:warning=Successfully embedded icon");
    Ok(())
}

#[cfg(target_os = "windows")]
fn convert_png_to_ico() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::BufWriter;

    // 检查ICO文件是否已存在且比PNG新
    if Path::new("assets/icon.ico").exists() {
        let png_meta = std::fs::metadata("assets/icon.png")?;
        let ico_meta = std::fs::metadata("assets/icon.ico")?;

        if ico_meta.modified()? >= png_meta.modified()? {
            println!("cargo:warning=ICO file is up to date");
            return Ok(());
        }
    }

    // 读取PNG文件
    let img = image::open("assets/icon.png")?;

    // 创建不同尺寸的图标
    let sizes = [16, 32, 48, 64];
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for &size in &sizes {
        let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8();

        let icon_image = ico::IconImage::from_rgba_data(size, size, rgba.into_raw());
        icon_dir.add_entry(ico::IconDirEntry::encode(&icon_image)?);
    }

    // 写入ICO文件
    let file = File::create("assets/icon.ico")?;
    let mut writer = BufWriter::new(file);
    icon_dir.write(&mut writer)?;

    println!("cargo:warning=Converted PNG to ICO");
    Ok(())
}

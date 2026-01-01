#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod hotkey_manager;
mod mouse_controller;


use app::AutoClickerApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let icon = load_icon();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([450.0, 600.0])
            .with_min_inner_size([400.0, 500.0])
            .with_resizable(true)
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Rust-AC",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Box::new(AutoClickerApp::new(cc))
        }),
    )
}

fn setup_fonts(ctx: &egui::Context) {
    use std::fs;

    let mut fonts = egui::FontDefinitions::default();

    // 尝试加载Windows系统中文字体
    let font_paths = [
        "C:/Windows/Fonts/msyh.ttc",     // 微软雅黑
        "C:/Windows/Fonts/simsun.ttc",   // 宋体
        "C:/Windows/Fonts/simhei.ttf",   // 黑体
    ];

    for (i, font_path) in font_paths.iter().enumerate() {
        if let Ok(font_data) = fs::read(font_path) {
            let font_name = format!("chinese_font_{}", i);
            fonts.font_data.insert(
                font_name.clone(),
                egui::FontData::from_owned(font_data),
            );

            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, font_name.clone());

            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, font_name);

            break; // 找到第一个可用字体就停止
        }
    }

    ctx.set_fonts(fonts);
}

fn load_icon() -> egui::IconData {
    // 尝试加载PNG图标文件
    if let Ok(icon_bytes) = std::fs::read("assets/icon.png") {
        if let Ok(image) = image::load_from_memory(&icon_bytes) {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();
            println!("成功加载图标: {}x{}", width, height);
            return egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            };
        } else {
            println!("图标解码失败");
        }
    } else {
        println!("图标文件不存在: assets/icon.png");
    }

    // 如果加载失败，创建一个默认图标
    println!("使用默认图标");
    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity(width * height * 4);

    for y in 0..height {
        for x in 0..width {
            if (x >= 8 && x < 24) && (y >= 8 && y < 24) {
                rgba.extend_from_slice(&[255, 100, 100, 255]); // 红色中心
            } else {
                rgba.extend_from_slice(&[100, 150, 255, 255]); // 蓝色边框
            }
        }
    }

    egui::IconData {
        rgba,
        width: width as u32,
        height: height as u32,
    }
}
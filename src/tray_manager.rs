use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder, Icon,
};
use std::sync::mpsc::{self, Receiver};

pub enum TrayEvent {
    ShowWindow,
    HideWindow,
    StartClicking,
    StopClicking,
    Quit,
}

pub struct TrayManager {
    tray_icon: TrayIcon,
    receiver: Receiver<MenuEvent>,
    show_item_id: tray_icon::menu::MenuId,
    hide_item_id: tray_icon::menu::MenuId,
    start_item_id: tray_icon::menu::MenuId,
    stop_item_id: tray_icon::menu::MenuId,
    quit_item_id: tray_icon::menu::MenuId,
}

impl TrayManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("开始初始化托盘管理器...");
        let (sender, receiver) = mpsc::channel();

        // 创建托盘菜单
        println!("创建托盘菜单...");
        let show_item = MenuItem::new("显示窗口", true, None);
        let hide_item = MenuItem::new("隐藏窗口", true, None);
        let start_item = MenuItem::new("切换连点", true, None);
        let stop_item = MenuItem::new("停止连点", true, None);
        let quit_item = MenuItem::new("退出程序", true, None);

        let show_item_id = show_item.id().clone();
        let hide_item_id = hide_item.id().clone();
        let start_item_id = start_item.id().clone();
        let stop_item_id = stop_item.id().clone();
        let quit_item_id = quit_item.id().clone();

        let menu = Menu::new();
        menu.append(&show_item).map_err(|e| format!("添加显示菜单项失败: {}", e))?;
        menu.append(&hide_item).map_err(|e| format!("添加隐藏菜单项失败: {}", e))?;
        menu.append(&PredefinedMenuItem::separator()).map_err(|e| format!("添加分隔符失败: {}", e))?;
        menu.append(&start_item).map_err(|e| format!("添加开始菜单项失败: {}", e))?;
        menu.append(&stop_item).map_err(|e| format!("添加停止菜单项失败: {}", e))?;
        menu.append(&PredefinedMenuItem::separator()).map_err(|e| format!("添加分隔符失败: {}", e))?;
        menu.append(&quit_item).map_err(|e| format!("添加退出菜单项失败: {}", e))?;

        // 设置菜单事件处理器
        println!("设置菜单事件处理器...");
        MenuEvent::set_event_handler(Some(move |event| {
            let _ = sender.send(event);
        }));

        // 创建图标
        println!("创建托盘图标...");
        let icon = Self::create_icon().map_err(|e| format!("创建图标失败: {}", e))?;

        // 创建托盘图标
        println!("构建托盘图标...");
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Rust-AC - 右键查看菜单")
            .with_icon(icon)
            .build()
            .map_err(|e| format!("构建托盘图标失败: {}", e))?;

        println!("托盘管理器初始化完成");
        Ok(Self {
            tray_icon,
            receiver,
            show_item_id,
            hide_item_id,
            start_item_id,
            stop_item_id,
            quit_item_id,
        })
    }

    fn create_icon() -> Result<Icon, Box<dyn std::error::Error>> {
        // 首先尝试加载ICO文件
        if std::path::Path::new("assets/icon.ico").exists() {
            println!("尝试加载托盘图标: assets/icon.ico");
            match Icon::from_path("assets/icon.ico", Some((32, 32))) {
                Ok(icon) => {
                    println!("ICO托盘图标加载成功");
                    return Ok(icon);
                }
                Err(e) => {
                    println!("ICO托盘图标加载失败: {}", e);
                }
            }
        }

        // 然后尝试加载PNG文件
        if std::path::Path::new("assets/icon.png").exists() {
            println!("尝试加载托盘图标: assets/icon.png");
            match Icon::from_path("assets/icon.png", Some((32, 32))) {
                Ok(icon) => {
                    println!("PNG托盘图标加载成功");
                    return Ok(icon);
                }
                Err(e) => {
                    println!("PNG托盘图标加载失败: {}", e);
                }
            }
        } else {
            println!("托盘图标文件不存在: assets/icon.png");
        }

        // 如果都加载失败，创建一个更明显的图标
        println!("使用默认托盘图标");
        let mut icon_data = Vec::new();
        for y in 0..32 {
            for x in 0..32 {
                // 创建一个更明显的图标 - 红色圆圈
                let center_x = 16;
                let center_y = 16;
                let distance = ((x as i32 - center_x).pow(2) + (y as i32 - center_y).pow(2)) as f32;
                let radius = 12.0;

                if distance <= radius * radius {
                    icon_data.extend_from_slice(&[255, 50, 50, 255]); // 红色圆圈
                } else {
                    icon_data.extend_from_slice(&[0, 0, 0, 0]); // 透明背景
                }
            }
        }

        Icon::from_rgba(icon_data, 32, 32).map_err(|e| e.into())
    }

    pub fn update_tooltip(&self, status: &str, clicks: u64) {
        let tooltip = format!("鼠标连点器 - {} (点击: {}次)", status, clicks);
        let _ = self.tray_icon.set_tooltip(Some(&tooltip));
    }

    pub fn check_events(&self) -> Option<TrayEvent> {
        if let Ok(event) = self.receiver.try_recv() {
            if event.id == self.show_item_id {
                return Some(TrayEvent::ShowWindow);
            } else if event.id == self.hide_item_id {
                return Some(TrayEvent::HideWindow);
            } else if event.id == self.start_item_id {
                return Some(TrayEvent::StartClicking);
            } else if event.id == self.stop_item_id {
                return Some(TrayEvent::StopClicking);
            } else if event.id == self.quit_item_id {
                return Some(TrayEvent::Quit);
            }
        }
        None
    }
}

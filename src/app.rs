use crate::config::{AppConfig, MouseButton, HotkeyConfig, IntervalMode};
use crate::hotkey_manager::{HotkeyAction, HotkeyManager};
use crate::mouse_controller::MouseController;

use eframe::egui;
use std::time::Instant;

pub struct AutoClickerApp {
    config: AppConfig,
    mouse_controller: MouseController,
    hotkey_manager: Option<HotkeyManager>,

    status_message: String,
    interval_input: String,
    cps_input: String,
    last_update: Instant,
    last_stats_update: Instant,
    hotkey_error: Option<String>,
    show_hotkey_settings: bool,
    temp_toggle_hotkey: HotkeyConfig,
    style_initialized: bool,
}

impl AutoClickerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = AppConfig::load();
        let interval_input = config.click_interval.to_string();
        let cps_input = config.cps_value.to_string();

        // 初始化热键管理器
        let (hotkey_manager, initial_hotkey_error) = match HotkeyManager::new() {
            Ok(mut manager) => {
                match manager.update_hotkeys(&config.toggle_hotkey) {
                    Ok(_) => (Some(manager), None),
                    Err(e) => {
                        eprintln!("热键注册失败: {}", e);
                        (Some(manager), Some(e))
                    }
                }
            }
            Err(e) => {
                eprintln!("热键管理器初始化失败: {}", e);
                (None, Some(format!("热键管理器初始化失败: {}", e)))
            }
        };

        Self {
            temp_toggle_hotkey: config.toggle_hotkey.clone(),
            config,
            mouse_controller: MouseController::new(),
            hotkey_manager,

            status_message: if initial_hotkey_error.is_some() {
                "就绪 (热键未注册)".to_string()
            } else {
                "就绪".to_string()
            },
            interval_input,
            cps_input,
            last_update: Instant::now(),
            last_stats_update: Instant::now(),
            hotkey_error: initial_hotkey_error,
            show_hotkey_settings: false,
            style_initialized: false,
        }
    }

    fn handle_hotkey_events(&mut self) {
        if let Some(ref mut hotkey_manager) = self.hotkey_manager {
            if let Some(action) = hotkey_manager.check_events(self.config.hold_mode) {
                match action {
                    HotkeyAction::Toggle => {
                        // 切换模式：按一次切换状态
                        if self.config.is_running {
                            self.stop_clicking();
                        } else {
                            self.start_clicking();
                        }
                    }
                    HotkeyAction::HoldStart => {
                        // 长按模式：按下开始
                        if !self.config.is_running {
                            self.start_clicking();
                        }
                    }
                    HotkeyAction::HoldStop => {
                        // 长按模式：松开停止
                        if self.config.is_running {
                            self.stop_clicking();
                        }
                    }
                }
            }
        }
    }



    fn start_clicking(&mut self) {
        // 根据当前模式更新配置
        match self.config.interval_mode {
            IntervalMode::Milliseconds => {
                let interval = self.config.click_interval;
                if interval > 0 {
                    let _ = self.config.save();
                } else {
                    self.status_message = "间隔时间必须大于0".to_string();
                    return;
                }
            }
            IntervalMode::CPS => {
                let cps = self.config.cps_value;
                if cps > 0 {
                    let _ = self.config.save();
                } else {
                    self.status_message = "CPS必须大于0".to_string();
                    return;
                }
            }
        }

        let effective_interval = self.config.get_effective_interval();
        self.mouse_controller
            .start_clicking(self.config.mouse_button, effective_interval);
        self.config.is_running = true;

        let mode_text = match self.config.interval_mode {
            IntervalMode::Milliseconds => format!("{}ms间隔", effective_interval),
            IntervalMode::CPS => format!("{} CPS", self.config.cps_value),
        };

        self.status_message = format!("正在连点 - {} ({})",
            self.config.mouse_button, mode_text);
    }

    fn stop_clicking(&mut self) {
        self.mouse_controller.stop_clicking();
        self.config.is_running = false;
        self.status_message = "已停止".to_string();
    }
}

impl eframe::App for AutoClickerApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // 提高热键检查频率，确保响应及时
        let now = Instant::now();
        if now.duration_since(self.last_update) >= std::time::Duration::from_millis(10) {
            self.handle_hotkey_events();
            self.last_update = now;
        }

        // 只初始化一次样式，避免每帧都克隆
        if !self.style_initialized {
            let mut style = (*ctx.style()).clone();
            style.spacing.button_padding = egui::vec2(12.0, 8.0);
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            ctx.set_style(style);
            self.style_initialized = true;
        }

        // 主窗口UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading("鼠标连点器");
                ui.add_space(5.0);
                ui.label("高效、简洁的自动点击工具");
            });

            ui.separator();
            ui.add_space(10.0);

            // 主要控制区域
            ui.group(|ui| {
                ui.set_min_width(350.0);

                // 鼠标按键选择
                ui.horizontal(|ui| {
                    ui.label("鼠标按键:");
                    if ui.radio_value(&mut self.config.mouse_button, MouseButton::Left, "左键").changed() {
                        let _ = self.config.save();
                    }
                    if ui.radio_value(&mut self.config.mouse_button, MouseButton::Right, "右键").changed() {
                        let _ = self.config.save();
                    }
                });

                ui.add_space(8.0);

                // 间隔模式选择
                ui.horizontal(|ui| {
                    ui.label("间隔模式:");
                    if ui.radio_value(&mut self.config.interval_mode, IntervalMode::Milliseconds, "毫秒").changed() {
                        let _ = self.config.save();
                    }
                    if ui.radio_value(&mut self.config.interval_mode, IntervalMode::CPS, "CPS").changed() {
                        let _ = self.config.save();
                    }
                });

                ui.add_space(5.0);

                // 间隔时间设置
                match self.config.interval_mode {
                    IntervalMode::Milliseconds => {
                        ui.horizontal(|ui| {
                            ui.label("点击间隔:");
                            if ui.add(egui::DragValue::new(&mut self.config.click_interval)
                                .speed(1.0)
                                .clamp_range(1..=10000)
                                .suffix(" ms")).changed() {
                                let _ = self.config.save();
                                self.interval_input = self.config.click_interval.to_string();
                            }

                            if ui.small_button("快速").clicked() {
                                self.config.click_interval = 50;
                                self.interval_input = "50".to_string();
                                let _ = self.config.save();
                                self.cps_input = self.config.cps_value.to_string();
                            }
                            if ui.small_button("中等").clicked() {
                                self.config.click_interval = 100;
                                self.interval_input = "100".to_string();
                                let _ = self.config.save();
                            }
                            if ui.small_button("慢速").clicked() {
                                self.config.click_interval = 500;
                                self.interval_input = "500".to_string();
                                let _ = self.config.save();
                            }
                        });
                    }
                    IntervalMode::CPS => {
                        ui.horizontal(|ui| {
                            ui.label("点击频率:");
                            if ui.add(egui::DragValue::new(&mut self.config.cps_value)
                                .speed(1.0)
                                .clamp_range(1..=1000)
                                .suffix(" CPS")).changed() {
                                let _ = self.config.save();
                            }

                            if ui.small_button("慢速").clicked() {
                                self.config.cps_value = 2;
                                self.cps_input = "2".to_string();
                                let _ = self.config.save();
                            }
                            if ui.small_button("中等").clicked() {
                                self.config.cps_value = 10;
                                self.cps_input = "10".to_string();
                                let _ = self.config.save();
                            }
                            if ui.small_button("快速").clicked() {
                                self.config.cps_value = 20;
                                self.cps_input = "20".to_string();
                                let _ = self.config.save();
                            }
                        });
                        
                        // 高 CPS 警告
                        if self.config.cps_value > 100 {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 165, 0),
                                format!("⚠ 高频率 ({} CPS) 可能影响系统性能", self.config.cps_value)
                            );
                        }
                    }
                }

                ui.add_space(8.0);

                // 触发模式设置
                ui.horizontal(|ui| {
                    ui.label("触发模式:");
                    if ui.checkbox(&mut self.config.hold_mode, "长按触发").changed() {
                        // 切换模式时重置热键状态
                        if let Some(ref mut hotkey_manager) = self.hotkey_manager {
                            hotkey_manager.reset_key_state();
                            let _ = hotkey_manager.update_hotkeys(&self.config.toggle_hotkey);
                        }
                        let _ = self.config.save();
                    }
                    ui.label(if self.config.hold_mode {
                        "(按住热键连点，松开停止)"
                    } else {
                        "(按一次开始，再按一次停止)"
                    });
                });

                ui.add_space(8.0);

                // 控制按钮
                ui.horizontal(|ui| {
                    let button_size = egui::vec2(100.0, 35.0);

                    if self.config.is_running {
                        if ui.add_sized(button_size, egui::Button::new("停止"))
                            .on_hover_text("停止自动点击")
                            .clicked() {
                            self.stop_clicking();
                        }
                    } else {
                        if ui.add_sized(button_size, egui::Button::new("开始"))
                            .on_hover_text("开始自动点击")
                            .clicked() {
                            self.start_clicking();
                        }
                    }

                    ui.add_space(10.0);

                    if ui.add_sized(button_size, egui::Button::new("热键设置"))
                        .on_hover_text("配置快捷键")
                        .clicked() {
                        self.show_hotkey_settings = true;
                    }
                });
            });

            ui.add_space(10.0);

            // 统计信息
            ui.group(|ui| {
                ui.label("统计信息");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("状态:");
                    if self.config.is_running {
                        ui.colored_label(egui::Color32::GREEN, "运行中");
                    } else {
                        ui.colored_label(egui::Color32::GRAY, "已停止");
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("总点击次数:");
                    ui.label(format!("{}", self.mouse_controller.get_click_count()));
                });

                if self.config.is_running {
                    ui.horizontal(|ui| {
                        ui.label("运行时间:");
                        if let Some(duration) = self.mouse_controller.get_running_time() {
                            ui.label(format!("{:.1}秒", duration.as_secs_f64()));
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("点击频率:");
                        ui.label(format!("{:.1} 次/秒", self.mouse_controller.get_cps()));
                    });
                }
            });

            ui.add_space(10.0);

            // 热键信息
            ui.group(|ui| {
                ui.label("快捷键");
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("切换:");
                    ui.code(self.config.toggle_hotkey.to_display_string());
                });

                if let Some(ref error) = self.hotkey_error {
                    ui.colored_label(egui::Color32::RED, format!("警告: {}", error));
                }
            });
        });

        // 热键设置窗口
        if self.show_hotkey_settings {
            self.show_hotkey_settings_window(ctx);
        }

        // 自动最小化到托盘
        if self.config.auto_minimize && self.config.is_running {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.config.window_visible = false;
        }

        // 智能重绘：平衡性能和响应性
        if self.config.is_running {
            // 运行时：限制统计更新频率
            if now.duration_since(self.last_stats_update) >= std::time::Duration::from_millis(500) {
                self.last_stats_update = now;
                ctx.request_repaint();
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(100));
            }
        } else if self.show_hotkey_settings {
            // 热键设置窗口打开时保持响应
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        } else {
            // 空闲时：保持较低频率以检测热键
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.stop_clicking();
        let _ = self.config.save();
    }
}

impl AutoClickerApp {
    fn show_hotkey_settings_window(&mut self, ctx: &egui::Context) {
        let mut apply_clicked = false;
        let mut cancel_clicked = false;
        let mut reset_clicked = false;

        egui::Window::new("热键设置")
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // 切换热键设置
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.strong("切换连点热键");
                            ui.add_space(5.0);

                            ui.label("修饰键 (可多选):");
                            Self::hotkey_modifier_ui(ui, &mut self.temp_toggle_hotkey.modifiers);

                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("主按键:");
                                Self::hotkey_key_ui(ui, &mut self.temp_toggle_hotkey.key);
                            });

                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label("预览:");
                                ui.code(self.temp_toggle_hotkey.to_display_string());
                            });
                        });
                    });

                    ui.add_space(15.0);

                    // 按钮区域
                    ui.horizontal(|ui| {
                        if ui.add_sized([80.0, 30.0], egui::Button::new("应用")).clicked() {
                            apply_clicked = true;
                        }

                        if ui.add_sized([80.0, 30.0], egui::Button::new("取消")).clicked() {
                            cancel_clicked = true;
                        }

                        if ui.add_sized([80.0, 30.0], egui::Button::new("重置")).clicked() {
                            reset_clicked = true;
                        }
                    });

                    // 错误信息
                    if let Some(ref error) = self.hotkey_error {
                        ui.add_space(10.0);
                        ui.colored_label(egui::Color32::RED, format!("警告: {}", error));
                    }

                    // 使用说明
                    ui.add_space(10.0);
                    ui.group(|ui| {
                        ui.label("使用说明:");
                        ui.label("• 可以选择多个修饰键组合");
                        ui.label("• 修饰键 + 主按键 = 完整热键");
                        ui.label("• 建议使用不常用的组合避免冲突");
                    });
                });
            });

        if apply_clicked {
            self.apply_hotkey_settings();
        }

        if cancel_clicked {
            self.temp_toggle_hotkey = self.config.toggle_hotkey.clone();
            self.show_hotkey_settings = false;
        }

        if reset_clicked {
            self.temp_toggle_hotkey = HotkeyConfig {
                modifiers: vec![],
                key: "F1".to_string(),
            };
        }
    }

    fn hotkey_modifier_ui(ui: &mut egui::Ui, modifiers: &mut Vec<String>) {
        let available_modifiers = ["Ctrl", "Alt", "Shift", "Win"];

        ui.horizontal(|ui| {
            for modifier in available_modifiers {
                let mut checked = modifiers.contains(&modifier.to_string());
                if ui.checkbox(&mut checked, modifier).changed() {
                    if checked {
                        if !modifiers.contains(&modifier.to_string()) {
                            modifiers.push(modifier.to_string());
                        }
                    } else {
                        modifiers.retain(|m| m != modifier);
                    }
                }
            }
        });
    }

    fn hotkey_key_ui(ui: &mut egui::Ui, key: &mut String) {
        let keys = [
            ("功能键", vec!["F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12"]),
            ("字母键", vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"]),
            ("导航键", vec!["Home", "End", "PageUp", "PageDown", "Insert", "Delete"]),
            ("锁定键", vec!["CapsLock", "NumLock", "ScrollLock"]),
            ("特殊键", vec!["Space", "Enter", "Esc", "Tab"]),
        ];

        // 使用当前key值作为ID的一部分来确保唯一性
        let combo_id = format!("hotkey_key_{}", key);
        egui::ComboBox::from_id_source(combo_id)
            .selected_text(key.as_str())
            .width(120.0)
            .show_ui(ui, |ui| {
                for (category, key_list) in keys {
                    ui.label(format!("--- {} ---", category));
                    for k in key_list {
                        ui.selectable_value(key, k.to_string(), k);
                    }
                    ui.separator();
                }
            });
    }

    fn apply_hotkey_settings(&mut self) {
        self.config.toggle_hotkey = self.temp_toggle_hotkey.clone();

        if let Some(ref mut hotkey_manager) = self.hotkey_manager {
            match hotkey_manager.update_hotkeys(&self.config.toggle_hotkey) {
                Ok(_) => {
                    self.hotkey_error = None;
                    self.show_hotkey_settings = false;
                    self.status_message = "热键设置已更新".to_string();
                }
                Err(e) => {
                    self.hotkey_error = Some(e);
                }
            }
        }
    }
}

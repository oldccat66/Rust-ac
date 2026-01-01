use serde::{Deserialize, Serialize};
use global_hotkey::hotkey::{Code, Modifiers};
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
}

impl Default for MouseButton {
    fn default() -> Self {
        MouseButton::Left
    }
}

impl std::fmt::Display for MouseButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MouseButton::Left => write!(f, "左键"),
            MouseButton::Right => write!(f, "右键"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifiers: vec!["Ctrl".to_string()],
            key: "F1".to_string(),
        }
    }
}

impl HotkeyConfig {
    pub fn to_display_string(&self) -> String {
        if self.modifiers.is_empty() {
            self.key.clone()
        } else {
            format!("{}+{}", self.modifiers.join("+"), self.key)
        }
    }

    pub fn to_global_hotkey(&self) -> Result<(Modifiers, Code), String> {
        let mut modifiers = Modifiers::empty();

        for modifier in &self.modifiers {
            match modifier.as_str() {
                "Ctrl" => modifiers |= Modifiers::CONTROL,
                "Alt" => modifiers |= Modifiers::ALT,
                "Shift" => modifiers |= Modifiers::SHIFT,
                "Win" => modifiers |= Modifiers::SUPER,
                _ => return Err(format!("未知修饰键: {}", modifier)),
            }
        }

        let code = match self.key.as_str() {
            "F1" => Code::F1,
            "F2" => Code::F2,
            "F3" => Code::F3,
            "F4" => Code::F4,
            "F5" => Code::F5,
            "F6" => Code::F6,
            "F7" => Code::F7,
            "F8" => Code::F8,
            "F9" => Code::F9,
            "F10" => Code::F10,
            "F11" => Code::F11,
            "F12" => Code::F12,
            "Space" => Code::Space,
            "Enter" => Code::Enter,
            "Esc" => Code::Escape,
            "Tab" => Code::Tab,
            "Home" => Code::Home,
            "End" => Code::End,
            "PageUp" => Code::PageUp,
            "PageDown" => Code::PageDown,
            "Insert" => Code::Insert,
            "Delete" => Code::Delete,
            "CapsLock" => Code::CapsLock,
            "NumLock" => Code::NumLock,
            "ScrollLock" => Code::ScrollLock,
            "A" => Code::KeyA,
            "B" => Code::KeyB,
            "C" => Code::KeyC,
            "D" => Code::KeyD,
            "E" => Code::KeyE,
            "F" => Code::KeyF,
            "G" => Code::KeyG,
            "H" => Code::KeyH,
            "I" => Code::KeyI,
            "J" => Code::KeyJ,
            "K" => Code::KeyK,
            "L" => Code::KeyL,
            "M" => Code::KeyM,
            "N" => Code::KeyN,
            "O" => Code::KeyO,
            "P" => Code::KeyP,
            "Q" => Code::KeyQ,
            "R" => Code::KeyR,
            "S" => Code::KeyS,
            "T" => Code::KeyT,
            "U" => Code::KeyU,
            "V" => Code::KeyV,
            "W" => Code::KeyW,
            "X" => Code::KeyX,
            "Y" => Code::KeyY,
            "Z" => Code::KeyZ,
            _ => return Err(format!("未知按键: {}", self.key)),
        };

        Ok((modifiers, code))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum IntervalMode {
    Milliseconds,
    CPS,
}

impl Default for IntervalMode {
    fn default() -> Self {
        IntervalMode::Milliseconds
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mouse_button: MouseButton,
    pub click_interval: u64,
    pub cps_value: u64,
    pub interval_mode: IntervalMode,
    pub is_running: bool,
    pub window_visible: bool,
    pub toggle_hotkey: HotkeyConfig,
    pub total_clicks: u64,
    pub auto_minimize: bool,
    pub hold_mode: bool, // true: 长按触发, false: 切换模式
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mouse_button: MouseButton::Left,
            click_interval: 100,
            cps_value: 10,
            interval_mode: IntervalMode::Milliseconds,
            is_running: false,
            window_visible: true,
            toggle_hotkey: HotkeyConfig {
                modifiers: vec![],
                key: "F1".to_string(),
            },
            total_clicks: 0,
            auto_minimize: false,
            hold_mode: false,
        }
    }
}

impl AppConfig {
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取配置文件路径
    pub fn get_config_path() -> Result<PathBuf, String> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| "无法获取用户主目录".to_string())?;

        let config_dir = home_dir.join(".config").join("Rust-ac");

        // 确保配置目录存在
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }

        Ok(config_dir.join("config.json"))
    }

    /// 从文件加载配置
    pub fn load() -> Self {
        match Self::load_from_file() {
            Ok(config) => {
                println!("配置加载成功");
                config
            }
            Err(e) => {
                println!("配置加载失败，使用默认配置: {}", e);
                Self::new()
            }
        }
    }

    fn load_from_file() -> Result<Self, String> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            return Err("配置文件不存在".to_string());
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;

        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))?;

        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::get_config_path()?;

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化配置失败: {}", e))?;

        fs::write(&config_path, json)
            .map_err(|e| format!("写入配置文件失败: {}", e))?;

        println!("配置已保存到: {}", config_path.display());
        Ok(())
    }
}

impl AppConfig {
    pub fn get_effective_interval(&self) -> u64 {
        match self.interval_mode {
            IntervalMode::Milliseconds => self.click_interval,
            IntervalMode::CPS => {
                if self.cps_value > 0 {
                    1000 / self.cps_value
                } else {
                    1000
                }
            }
        }
    }
}



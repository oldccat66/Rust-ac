use crate::config::HotkeyConfig;
use global_hotkey::{
    hotkey::HotKey,
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use std::sync::mpsc::{self, Receiver};

#[cfg(windows)]
use winapi::um::winuser::GetAsyncKeyState;

pub enum HotkeyAction {
    Toggle,
    HoldStart,
    HoldStop,
}

pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    receiver: Receiver<GlobalHotKeyEvent>,
    toggle_hotkey: Option<HotKey>,
    toggle_hotkey_id: Option<u32>,
    is_key_pressed: bool,
    current_hotkey: Option<HotkeyConfig>,
    last_poll_time: std::time::Instant,
}

impl HotkeyManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let manager = GlobalHotKeyManager::new()?;
        let (sender, receiver) = mpsc::channel();

        GlobalHotKeyEvent::set_event_handler(Some(move |event| {
            let _ = sender.send(event);
        }));

        Ok(Self {
            manager,
            receiver,
            toggle_hotkey: None,
            toggle_hotkey_id: None,
            is_key_pressed: false,
            current_hotkey: None,
            last_poll_time: std::time::Instant::now(),
        })
    }

    pub fn update_hotkeys(
        &mut self,
        toggle_config: &HotkeyConfig,
    ) -> Result<(), String> {
        // 先注销旧的热键
        if let Some(old_hotkey) = self.toggle_hotkey.take() {
            if let Err(e) = self.manager.unregister(old_hotkey) {
                eprintln!("注销旧热键失败: {}", e);
            }
        }
        self.toggle_hotkey_id = None;
        
        // 重置按键状态
        self.reset_key_state();

        // 注册新的切换热键
        match toggle_config.to_global_hotkey() {
            Ok((modifiers, code)) => {
                let hotkey = HotKey::new(Some(modifiers), code);
                match self.manager.register(hotkey) {
                    Ok(_) => {
                        self.toggle_hotkey_id = Some(hotkey.id());
                        self.toggle_hotkey = Some(hotkey);
                        self.current_hotkey = Some(toggle_config.clone());
                        Ok(())
                    }
                    Err(e) => {
                        // 提供更友好的错误信息
                        let error_msg = format!(
                            "热键 {} 已被占用，请尝试其他组合\n提示：可能与系统快捷键或其他应用冲突",
                            toggle_config.to_display_string()
                        );
                        Err(error_msg)
                    }
                }
            }
            Err(e) => Err(format!("热键配置错误: {}", e)),
        }
    }

    /// 重置按键状态，用于模式切换时清除旧状态
    pub fn reset_key_state(&mut self) {
        self.is_key_pressed = false;
        // 清空事件队列
        while self.receiver.try_recv().is_ok() {}
    }

    pub fn check_events(&mut self, hold_mode: bool) -> Option<HotkeyAction> {
        if hold_mode {
            // 长按模式：使用轮询检测按键状态
            let now = std::time::Instant::now();
            if now.duration_since(self.last_poll_time) < std::time::Duration::from_millis(10) {
                // 提高轮询频率到 100Hz，确保响应及时
                return None;
            }
            self.last_poll_time = now;
            
            // 清空事件队列防止堆积
            while self.receiver.try_recv().is_ok() {}
            
            return self.check_key_hold_state();
        }

        // 切换模式：事件驱动 + 轮询双保险
        // 先检查事件队列
        let mut event_triggered = false;
        while let Ok(event) = self.receiver.try_recv() {
            if Some(event.id) == self.toggle_hotkey_id {
                event_triggered = true;
            }
        }
        
        if event_triggered {
            // 事件触发时，检查是否是新的按下（防止重复触发）
            let pressed_now = self.is_key_currently_pressed();
            if pressed_now && !self.is_key_pressed {
                self.is_key_pressed = true;
                return Some(HotkeyAction::Toggle);
            }
        }

        // 如果事件系统失效，使用轮询作为后备
        #[cfg(windows)]
        {
            let now = std::time::Instant::now();
            if now.duration_since(self.last_poll_time) >= std::time::Duration::from_millis(50) {
                self.last_poll_time = now;
                
                let pressed_now = self.is_key_currently_pressed();
                if pressed_now && !self.is_key_pressed {
                    // 按下边沿：触发切换
                    self.is_key_pressed = true;
                    return Some(HotkeyAction::Toggle);
                } else if !pressed_now && self.is_key_pressed {
                    // 松开：重置状态
                    self.is_key_pressed = false;
                }
            }
        }

        None
    }

    fn check_key_hold_state(&mut self) -> Option<HotkeyAction> {
        // 使用Windows API检查按键状态
        let is_currently_pressed = self.is_key_currently_pressed();

        if is_currently_pressed && !self.is_key_pressed {
            // 按键刚被按下
            self.is_key_pressed = true;
            return Some(HotkeyAction::HoldStart);
        } else if !is_currently_pressed && self.is_key_pressed {
            // 按键刚被松开
            self.is_key_pressed = false;
            return Some(HotkeyAction::HoldStop);
        }

        None
    }

    #[cfg(windows)]
    fn is_key_currently_pressed(&self) -> bool {
        if let Some(ref hotkey_config) = self.current_hotkey {
            // 检查主按键
            let key_pressed = match hotkey_config.key.as_str() {
                "F1" => unsafe { GetAsyncKeyState(0x70) < 0 },
                "F2" => unsafe { GetAsyncKeyState(0x71) < 0 },
                "F3" => unsafe { GetAsyncKeyState(0x72) < 0 },
                "F4" => unsafe { GetAsyncKeyState(0x73) < 0 },
                "F5" => unsafe { GetAsyncKeyState(0x74) < 0 },
                "F6" => unsafe { GetAsyncKeyState(0x75) < 0 },
                "F7" => unsafe { GetAsyncKeyState(0x76) < 0 },
                "F8" => unsafe { GetAsyncKeyState(0x77) < 0 },
                "F9" => unsafe { GetAsyncKeyState(0x78) < 0 },
                "F10" => unsafe { GetAsyncKeyState(0x79) < 0 },
                "F11" => unsafe { GetAsyncKeyState(0x7A) < 0 },
                "F12" => unsafe { GetAsyncKeyState(0x7B) < 0 },
                "Space" => unsafe { GetAsyncKeyState(0x20) < 0 },
                "Enter" => unsafe { GetAsyncKeyState(0x0D) < 0 },
                "Esc" => unsafe { GetAsyncKeyState(0x1B) < 0 },
                "Tab" => unsafe { GetAsyncKeyState(0x09) < 0 },
                key if key.len() == 1 => {
                    let ch = key.chars().next().unwrap().to_ascii_uppercase();
                    if ch.is_ascii_alphabetic() {
                        unsafe { GetAsyncKeyState(ch as i32) < 0 }
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if !key_pressed {
                return false;
            }

            // 检查修饰键（兼容左右键）
            for modifier in &hotkey_config.modifiers {
                let modifier_pressed = match modifier.as_str() {
                    // Ctrl: VK_CONTROL, VK_LCONTROL(0xA2), VK_RCONTROL(0xA3)
                    "Ctrl" => unsafe {
                        (GetAsyncKeyState(0x11) < 0) || (GetAsyncKeyState(0xA2) < 0) || (GetAsyncKeyState(0xA3) < 0)
                    },
                    // Alt: VK_MENU, VK_LMENU(0xA4), VK_RMENU(0xA5)
                    "Alt" => unsafe {
                        (GetAsyncKeyState(0x12) < 0) || (GetAsyncKeyState(0xA4) < 0) || (GetAsyncKeyState(0xA5) < 0)
                    },
                    // Shift: VK_SHIFT, VK_LSHIFT(0xA0), VK_RSHIFT(0xA1)
                    "Shift" => unsafe {
                        (GetAsyncKeyState(0x10) < 0) || (GetAsyncKeyState(0xA0) < 0) || (GetAsyncKeyState(0xA1) < 0)
                    },
                    // Win: VK_LWIN(0x5B), VK_RWIN(0x5C)
                    "Win" => unsafe {
                        (GetAsyncKeyState(0x5B) < 0) || (GetAsyncKeyState(0x5C) < 0)
                    },
                    _ => false,
                };
                if !modifier_pressed {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }

    #[cfg(not(windows))]
    fn is_key_currently_pressed(&self) -> bool {
        false
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        // 程序退出时注销所有热键
        if let Some(hotkey) = self.toggle_hotkey.take() {
            let _ = self.manager.unregister(hotkey);
        }
    }
}

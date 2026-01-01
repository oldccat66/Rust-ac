use crate::config::MouseButton;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    mouse_event, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP,
};

#[cfg(windows)]
use windows::Win32::System::Threading::{
    GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_BELOW_NORMAL,
};

pub struct MouseController {
    is_running: Arc<AtomicBool>,
    click_count: Arc<AtomicU64>,
    handles: Vec<thread::JoinHandle<()>>,
    start_time: Option<Instant>,
}

impl MouseController {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            click_count: Arc::new(AtomicU64::new(0)),
            handles: Vec::new(),
            start_time: None,
        }
    }

    pub fn start_clicking(&mut self, button: MouseButton, interval_ms: u64) {
        if self.is_running.load(Ordering::Relaxed) {
            return;
        }

        self.is_running.store(true, Ordering::Relaxed);
        self.click_count.store(0, Ordering::Relaxed);
        self.start_time = Some(Instant::now());

        // 防止除零
        if interval_ms == 0 {
            return;
        }

        // 根据 CPS 决定线程数量
        let thread_count = if interval_ms >= 20 {
            1 // <= 50 CPS: 单线程足够
        } else if interval_ms >= 5 {
            2 // 50-200 CPS: 2 线程
        } else if interval_ms >= 2 {
            4 // 200-500 CPS: 4 线程
        } else {
            8 // > 500 CPS: 8 线程
        };

        let is_running = Arc::clone(&self.is_running);
        let click_count = Arc::clone(&self.click_count);

        // 创建多个点击线程
        for thread_id in 0..thread_count {
            let is_running = Arc::clone(&is_running);
            let click_count = Arc::clone(&click_count);
            
            let handle = thread::spawn(move || {
                // 设置线程优先级
                #[cfg(windows)]
                unsafe {
                    let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);
                }
                
                let interval = Duration::from_millis(interval_ms * thread_count as u64);
                let offset = Duration::from_millis(interval_ms * thread_id as u64);
                let mut next_click = Instant::now() + offset;
                
                while is_running.load(Ordering::Relaxed) {
                    let now = Instant::now();
                    
                    if now >= next_click {
                        Self::simulate_click(button);
                        click_count.fetch_add(1, Ordering::Relaxed);
                        next_click += interval;
                        
                        // 防止时间漂移
                        if next_click < now {
                            next_click = now + interval;
                        }
                    }
                    
                    // 智能睡眠
                    let time_until_next = next_click.saturating_duration_since(Instant::now());
                    if time_until_next > Duration::from_millis(2) {
                        thread::sleep(time_until_next - Duration::from_millis(1));
                    } else if time_until_next > Duration::ZERO {
                        // 短暂忙等待提高精度
                        let spin_until = Instant::now() + time_until_next;
                        while Instant::now() < spin_until && is_running.load(Ordering::Relaxed) {
                            std::hint::spin_loop();
                        }
                    }
                }
            });
            
            self.handles.push(handle);
        }
    }

    pub fn stop_clicking(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);

        // 等待所有线程结束
        for handle in self.handles.drain(..) {
            let _ = handle.join();
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    pub fn get_click_count(&self) -> u64 {
        self.click_count.load(Ordering::Relaxed)
    }

    pub fn get_running_time(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }

    pub fn get_cps(&self) -> f64 {
        if let Some(duration) = self.get_running_time() {
            let seconds = duration.as_secs_f64();
            if seconds > 0.0 {
                return self.get_click_count() as f64 / seconds;
            }
        }
        0.0
    }

    fn simulate_click(button: MouseButton) {
        unsafe {
            match button {
                MouseButton::Left => {
                    mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
                    mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
                }
                MouseButton::Right => {
                    mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0, 0);
                    mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0, 0);
                }
            }
        }
    }
}

impl Drop for MouseController {
    fn drop(&mut self) {
        self.stop_clicking();
    }
}

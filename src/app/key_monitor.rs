use std::{
    sync::{Arc, Mutex},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use rdev::{EventType, listen};
use tracing::{error, info};

#[derive(Clone)]
pub(crate) struct LogEntry {
    pub(crate) time: String,
    pub(crate) key: String,
    pub(crate) process: String,
    pub(crate) window: String,
}

pub(crate) struct KeyMonitor {
    is_active: Arc<Mutex<bool>>,
    entries: Arc<Mutex<Vec<LogEntry>>>,
    thread_started: bool,
}

const MAX_ENTRIES: usize = 500;

impl Default for KeyMonitor {
    fn default() -> Self {
        Self {
            is_active: Arc::new(Mutex::new(false)),
            entries: Arc::new(Mutex::new(Vec::new())),
            thread_started: false,
        }
    }
}

impl KeyMonitor {
    pub(crate) fn is_active(&self) -> bool {
        *self.is_active.lock().expect("mutex poisoned")
    }

    pub(crate) fn toggle(&mut self) {
        let mut active = self.is_active.lock().expect("mutex poisoned");
        *active = !*active;
        let now_active = *active;
        drop(active);

        if now_active && !self.thread_started {
            self.thread_started = true;
            self.spawn_listener();
        }
    }

    pub(crate) fn clear(&self) {
        self.entries.lock().expect("mutex poisoned").clear();
    }

    pub(crate) fn snapshot(&self) -> Vec<LogEntry> {
        self.entries.lock().expect("mutex poisoned").clone()
    }

    pub(crate) fn entry_count(&self) -> usize {
        self.entries.lock().expect("mutex poisoned").len()
    }

    fn spawn_listener(&self) {
        let is_active = Arc::clone(&self.is_active);
        let entries = Arc::clone(&self.entries);

        thread::spawn(move || {
            info!("Поток монитора клавиш запущен");

            if let Err(e) = listen(move |event| {
                if let EventType::KeyPress(key) = event.event_type {
                    if !*is_active.lock().expect("mutex poisoned") {
                        return;
                    }

                    let (process, window) = active_window_info();
                    let entry = LogEntry {
                        time: utc_time_str(),
                        key: format!("{:?}", &key),
                        process,
                        window,
                    };

                    let mut log = entries.lock().expect("mutex poisoned");
                    if log.len() >= MAX_ENTRIES {
                        log.remove(0);
                    }
                    log.push(entry);
                }
            }) {
                error!("rdev listen завершился с ошибкой: {:?}", e);
            }
        });
    }
}

fn utc_time_str() -> String {
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let s = d.as_secs();
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        (s / 3600) % 24,
        (s % 3600) / 60,
        s % 60,
        d.subsec_millis()
    )
}

#[cfg(windows)]
fn active_window_info() -> (String, String) {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{
            OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
        },
        UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId},
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return ("—".into(), "—".into());
        }

        // Заголовок окна
        let mut buf = [0u16; 256];
        let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        let title = if len > 0 {
            String::from_utf16_lossy(&buf[..len as usize]).to_string()
        } else {
            "—".into()
        };

        // Имя процесса
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid as *mut _);

        let hproc = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        let process = if !hproc.is_null() {
            let mut name_buf = [0u16; 260];
            let mut size = name_buf.len() as u32;
            // dwFlags = 0 → Win32-путь
            let ok = QueryFullProcessImageNameW(hproc, 0, name_buf.as_mut_ptr(), &mut size);
            CloseHandle(hproc);
            if ok != 0 && size > 0 {
                let full = String::from_utf16_lossy(&name_buf[..size as usize]);
                std::path::Path::new(&full)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "—".into())
            } else {
                "—".into()
            }
        } else {
            "—".into()
        };

        (process, title)
    }
}

#[cfg(not(windows))]
fn active_window_info() -> (String, String) {
    ("—".into(), "—".into())
}

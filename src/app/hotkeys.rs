use std::collections::HashMap;

use anyhow::Result;
use global_hotkey::{
    GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};
use tracing::{debug, error, warn};

pub(crate) struct HotKeys {
    pub(crate) event_map: HashMap<u32, HotKeyEvent>,
    pub(crate) _manager: GlobalHotKeyManager,
    registered_ids: HashMap<HotKeyEvent, u32>,
}

impl HotKeys {
    pub fn new(set_position_key: &str, toggle_key: &str) -> Result<Self> {
        let manager = GlobalHotKeyManager::new()?;
        let mut event_map = HashMap::new();
        let mut registered_ids = HashMap::new();

        // Регистрация клавиши установки позиции
        if let Some((mods, code)) = parse_hotkey(set_position_key) {
            match register_key(&manager, mods, code, HotKeyEvent::SetPosition) {
                Ok((id, event)) => {
                    event_map.insert(id, event);
                    registered_ids.insert(event, id);
                    debug!(
                        "Зарегистрирована клавиша установки позиции: {}",
                        set_position_key
                    );
                }
                Err(e) => {
                    warn!(
                        "Не удалось зарегистрировать клавишу {}: {}",
                        set_position_key, e
                    );
                }
            }
        } else {
            warn!(
                "Неверный формат клавиши установки позиции: {}",
                set_position_key
            );
        }

        // Регистрация клавиши переключения
        if let Some((mods, code)) = parse_hotkey(toggle_key) {
            match register_key(&manager, mods, code, HotKeyEvent::ToggleClicker) {
                Ok((id, event)) => {
                    event_map.insert(id, event);
                    registered_ids.insert(event, id);
                    debug!("Зарегистрирована клавиша переключения: {}", toggle_key);
                }
                Err(e) => {
                    warn!("Не удалось зарегистрировать клавишу {}: {}", toggle_key, e);
                }
            }
        } else {
            warn!("Неверный формат клавиши переключения: {}", toggle_key);
        }

        Ok(Self {
            event_map,
            _manager: manager,
            registered_ids,
        })
    }

    pub fn get_event(&self, id: u32) -> Option<&HotKeyEvent> {
        self.event_map.get(&id)
    }

    pub fn update_keys(&mut self, set_position_key: &str, toggle_key: &str) -> Result<()> {
        debug!("Обновление горячих клавиш");

        // Сохраняем старые HotKey объекты для отмены регистрации
        let mut old_hotkeys = Vec::new();

        for &event in self.event_map.values() {
            let key_str = match event {
                HotKeyEvent::SetPosition => set_position_key,
                HotKeyEvent::ToggleClicker => toggle_key,
            };

            if let Some((mods, code)) = parse_hotkey(key_str) {
                let hotkey = HotKey::new(mods, code);
                old_hotkeys.push(hotkey);
            }
        }

        // Отменяем регистрацию старых клавиш
        for hotkey in old_hotkeys {
            if let Err(e) = self._manager.unregister(hotkey) {
                warn!("Не удалось отменить регистрацию клавиши: {}", e);
            }
        }

        self.event_map.clear();
        self.registered_ids.clear();

        // Регистрируем новые клавиши
        if let Some((mods, code)) = parse_hotkey(set_position_key) {
            match register_key(&self._manager, mods, code, HotKeyEvent::SetPosition) {
                Ok((id, event)) => {
                    self.event_map.insert(id, event);
                    self.registered_ids.insert(event, id);
                }
                Err(e) => error!("Ошибка регистрации клавиши {}: {}", set_position_key, e),
            }
        }

        if let Some((mods, code)) = parse_hotkey(toggle_key) {
            match register_key(&self._manager, mods, code, HotKeyEvent::ToggleClicker) {
                Ok((id, event)) => {
                    self.event_map.insert(id, event);
                    self.registered_ids.insert(event, id);
                }
                Err(e) => error!("Ошибка регистрации клавиши {}: {}", toggle_key, e),
            }
        }

        Ok(())
    }
}

fn register_key(
    manager: &GlobalHotKeyManager,
    modifiers: Option<Modifiers>,
    code: Code,
    event: HotKeyEvent,
) -> Result<(u32, HotKeyEvent)> {
    let hotkey = HotKey::new(modifiers, code);
    manager.register(hotkey)?;
    Ok((hotkey.id, event))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotKeyEvent {
    ToggleClicker,
    SetPosition,
}

// Парсинг строки вида "Ctrl+Shift+F6" или "F7"
fn parse_hotkey(s: &str) -> Option<(Option<Modifiers>, Code)> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();

    if parts.is_empty() {
        return None;
    }

    let mut modifiers = Modifiers::empty();
    let key_part = parts.last()?;

    // Обработка модификаторов
    for part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "shift" => modifiers |= Modifiers::SHIFT,
            "alt" => modifiers |= Modifiers::ALT,
            "super" | "win" | "meta" => modifiers |= Modifiers::SUPER,
            _ => {}
        }
    }

    // Парсинг основной клавиши
    let code = match key_part.to_lowercase().as_str() {
        "f1" => Code::F1,
        "f2" => Code::F2,
        "f3" => Code::F3,
        "f4" => Code::F4,
        "f5" => Code::F5,
        "f6" => Code::F6,
        "f7" => Code::F7,
        "f8" => Code::F8,
        "f9" => Code::F9,
        "f10" => Code::F10,
        "f11" => Code::F11,
        "f12" => Code::F12,
        "a" => Code::KeyA,
        "b" => Code::KeyB,
        "c" => Code::KeyC,
        "d" => Code::KeyD,
        "e" => Code::KeyE,
        "f" => Code::KeyF,
        "g" => Code::KeyG,
        "h" => Code::KeyH,
        "i" => Code::KeyI,
        "j" => Code::KeyJ,
        "k" => Code::KeyK,
        "l" => Code::KeyL,
        "m" => Code::KeyM,
        "n" => Code::KeyN,
        "o" => Code::KeyO,
        "p" => Code::KeyP,
        "q" => Code::KeyQ,
        "r" => Code::KeyR,
        "s" => Code::KeyS,
        "t" => Code::KeyT,
        "u" => Code::KeyU,
        "v" => Code::KeyV,
        "w" => Code::KeyW,
        "x" => Code::KeyX,
        "y" => Code::KeyY,
        "z" => Code::KeyZ,
        "0" => Code::Digit0,
        "1" => Code::Digit1,
        "2" => Code::Digit2,
        "3" => Code::Digit3,
        "4" => Code::Digit4,
        "5" => Code::Digit5,
        "6" => Code::Digit6,
        "7" => Code::Digit7,
        "8" => Code::Digit8,
        "9" => Code::Digit9,
        "space" => Code::Space,
        "enter" => Code::Enter,
        "tab" => Code::Tab,
        "escape" | "esc" => Code::Escape,
        "backspace" => Code::Backspace,
        "insert" => Code::Insert,
        "delete" => Code::Delete,
        "home" => Code::Home,
        "end" => Code::End,
        "pageup" => Code::PageUp,
        "pagedown" => Code::PageDown,
        _ => return None,
    };

    let mods = if modifiers.is_empty() {
        None
    } else {
        Some(modifiers)
    };

    Some((mods, code))
}

pub fn validate_hotkey(s: &str) -> bool {
    parse_hotkey(s).is_some()
}

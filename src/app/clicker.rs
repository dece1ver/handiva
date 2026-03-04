use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use enigo::{Button, Coordinate, Direction, Enigo, Mouse, Settings};
use tracing::{debug, error, info, warn};

pub(crate) struct Clicker {
    target_x: Option<i32>,
    target_y: Option<i32>,
    interval: u64,
    is_working: Arc<Mutex<bool>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl Default for Clicker {
    fn default() -> Self {
        Self {
            target_x: None,
            target_y: None,
            interval: 500u64,
            is_working: Arc::new(Mutex::new(false)),
            thread_handle: None,
        }
    }
}

impl Clicker {
    pub(crate) fn set_interval<T: Into<u64>>(&mut self, value: T) {
        let new_interval = value.into();
        debug!("Установка интервала кликера: {} мс", new_interval);
        self.interval = new_interval;
    }

    pub(crate) fn set_target<T: Into<i32>>(&mut self, x: T, y: T) {
        let x_val = x.into();
        let y_val = y.into();
        debug!("Установка цели кликера: x={}, y={}", x_val, y_val);
        self.target_x = Some(x_val);
        self.target_y = Some(y_val);
    }

    pub(crate) fn target(&self) -> String {
        match (self.target_x, self.target_y) {
            (Some(x), Some(y)) => format!("X: {:4}  Y: {:4}", x, y),
            _ => "-".to_owned(),
        }
    }

    pub(crate) fn is_working(&self) -> bool {
        *self.is_working.lock().unwrap()
    }

    pub(crate) fn toggle(&mut self) {
        let mut is_working = self.is_working.lock().unwrap();

        if !*is_working {
            if self.target_x.is_none() || self.target_y.is_none() {
                warn!("Попытка запуска кликера без установленной цели");
                return;
            }

            info!("Запуск кликера");
            *is_working = true;
            drop(is_working);

            let is_working_clone = Arc::clone(&self.is_working);
            let target_x = self.target_x.unwrap();
            let target_y = self.target_y.unwrap();
            let interval = self.interval;

            let handle = thread::spawn(move || {
                debug!("Рабочий поток кликера запущен");
                let mut enigo = match Enigo::new(&Settings::default()) {
                    Ok(e) => e,
                    Err(err) => {
                        error!("Не удалось инициализировать Enigo в потоке: {}", err);
                        return;
                    }
                };

                let mut click_count = 0u64;

                loop {
                    {
                        let working = is_working_clone.lock().unwrap();
                        if !*working {
                            info!("Остановка кликера, выполнено кликов: {}", click_count);
                            break;
                        }
                    }

                    if let Err(e) = enigo.move_mouse(target_x, target_y, Coordinate::Abs) {
                        error!("Ошибка перемещения мыши: {}", e);
                    }

                    if let Err(e) = enigo.button(Button::Left, Direction::Click) {
                        error!("Ошибка клика: {}", e);
                    }

                    click_count += 1;

                    if click_count.is_multiple_of(100) {
                        debug!("Выполнено кликов: {}", click_count);
                    }

                    thread::sleep(Duration::from_millis(interval));
                }

                debug!("Рабочий поток кликера завершён");
            });

            self.thread_handle = Some(handle);
        } else {
            info!("Остановка кликера");
            *is_working = false;
        }
    }

    pub(crate) fn stop(&mut self) {
        debug!("Принудительная остановка кликера");
        *self.is_working.lock().unwrap() = false;

        if let Some(handle) = self.thread_handle.take()
            && let Err(e) = handle.join()
        {
            error!("Ошибка при завершении потока кликера: {:?}", e);
        }
    }
}

impl Drop for Clicker {
    fn drop(&mut self) {
        debug!("Уничтожение объекта Clicker");
        self.stop();
    }
}

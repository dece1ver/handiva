use std::{collections::HashSet, time::Duration};

use enigo::{Enigo, Mouse, Settings};
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use tracing::{debug, error, info, warn};

use crate::app::{
    clicker::Clicker,
    config::Config,
    hotkeys::{HotKeyEvent, HotKeys},
    icon_packer::IcoImage,
    key_monitor::KeyMonitor,
};

use anyhow::Result;

mod clicker;
mod config;
mod hotkeys;
mod icon_packer;
mod key_monitor;
mod tabs;
mod utils;

pub(crate) struct App {
    config: Config,
    clicker: Clicker,
    active_tab: Tab,
    ico_images: Vec<IcoImage>,
    ico_extra_sizes: HashSet<u32>,
    key_monitor: KeyMonitor,
    status: String,
    enigo: Enigo,
    interval_input: String,
    hotkeys: HotKeys,
    temp_set_position: String,
    temp_toggle: String,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Result<Self> {
        info!("Инициализация приложения");

        let config = Config::load().map_err(|e| {
            error!("Не удалось загрузить конфигурацию: {}", e);
            e
        })?;

        _cc.egui_ctx
            .send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                if config.always_on_top {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                },
            ));

        let hotkeys =
            HotKeys::new(&config.hotkeys.set_position, &config.hotkeys.toggle).map_err(|e| {
                error!("Не удалось инициализировать горячие клавиши: {}", e);
                e
            })?;
        let interval_input = config.default_interval.to_string();
        let enigo = Enigo::new(&Settings::default()).map_err(|e| {
            error!("Не удалось инициализировать Enigo: {}", e);
            anyhow::anyhow!("Ошибка инициализации Enigo: {}", e)
        })?;

        let mut clicker = Clicker::default();
        clicker.set_interval(config.default_interval);

        info!("Приложение успешно инициализировано");
        debug!("Начальная конфигурация: {:?}", config);

        Ok(Self {
            temp_set_position: config.hotkeys.set_position.clone(),
            temp_toggle: config.hotkeys.toggle.clone(),
            config,
            clicker,
            active_tab: Tab::Clicker,
            ico_images: Vec::new(),
            ico_extra_sizes: HashSet::new(),
            key_monitor: KeyMonitor::default(),
            status: "Готов к работе".to_string(),
            enigo,
            interval_input,
            hotkeys,
        })
    }
}

#[derive(PartialEq)]
enum Tab {
    Clicker,
    KeyMonitor,
    IconPacker,
    Settings,
    About,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state != HotKeyState::Released {
                continue;
            }
            if let Some(hotkey_event) = self.hotkeys.get_event(event.id) {
                match hotkey_event {
                    HotKeyEvent::ToggleClicker => {
                        debug!("Горячая клавиша: переключение кликера");
                        self.clicker.toggle();
                        self.status = if self.clicker.is_working() {
                            info!("Кликер запущен");
                            "Кликер запущен".into()
                        } else {
                            info!("Кликер остановлен");
                            "Кликер остановлен".into()
                        };
                    }
                    HotKeyEvent::SetPosition => {
                        debug!("Горячая клавиша: установка позиции");
                        if let Ok((x, y)) = self.enigo.location() {
                            self.clicker.set_target(x, y);
                            self.status = format!("Позиция установлена: {}, {}", x, y);
                            info!("Позиция установлена: x={}, y={}", x, y);
                        } else {
                            warn!("Не удалось получить координаты курсора");
                        }
                    }
                }
            }
        }

        // self.top_panel(ctx);
        self.side_menu(ctx);
        self.status_bar(ctx);
        self.central_content(ctx);

        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

impl App {
    // fn top_panel(&mut self, ctx: &egui::Context) {
    //     TopBottomPanel::top("top").show(ctx, |ui| {
    //         ui.horizontal(|ui| {
    //             ui.label("Top panel");
    //         });
    //     });
    // }

    fn side_menu(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("menu")
            .resizable(true)
            .default_width(130.0)
            .min_width(130.0)
            .max_width(200.0)
            .show(ctx, |ui| {
                if ui
                    .selectable_label(self.active_tab == Tab::Clicker, "Кликер")
                    .clicked()
                {
                    self.active_tab = Tab::Clicker;
                    self.status = "Открыт кликер".into();
                    debug!("Переключение на вкладку: Кликер");
                }
                if ui
                    .selectable_label(self.active_tab == Tab::KeyMonitor, "Монитор клавиш")
                    .clicked()
                {
                    self.active_tab = Tab::KeyMonitor;
                    self.status = "Открыт монитор клавиш".into();
                }
                if ui
                    .selectable_label(self.active_tab == Tab::IconPacker, "Сборщик иконок")
                    .clicked()
                {
                    self.active_tab = Tab::IconPacker;
                    self.status = "Открыт сборщик иконок".into();
                }
                if ui
                    .selectable_label(self.active_tab == Tab::Settings, "Настройки")
                    .clicked()
                {
                    self.active_tab = Tab::Settings;
                    self.status = "Открыты настройки".into();
                    debug!("Переключение на вкладку: Настройки");
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.spacing_mut().item_spacing.y = -1.0;
                    if ui
                        .selectable_label(self.active_tab == Tab::About, "О программе")
                        .clicked()
                    {
                        self.active_tab = Tab::About;
                        self.status = "Раздел About".into();
                    }
                    ui.separator();
                })
            });
    }

    fn central_content(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| match self.active_tab {
            Tab::Clicker => tabs::clicker::draw(self, ui),
            Tab::KeyMonitor => tabs::key_monitor::draw(self, ui),
            Tab::IconPacker => tabs::icon_packer::draw(self, ui, ctx),
            Tab::Settings => tabs::settings::draw(self, ui, ctx),
            Tab::About => tabs::about::draw(self, ui),
        });
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.monospace(&self.status);
            });
        });
    }
}

impl Drop for App {
    fn drop(&mut self) {
        info!("Завершение работы приложения");
        if let Err(e) = self.config.save() {
            error!("Не удалось сохранить конфигурацию при завершении: {}", e);
        }
    }
}

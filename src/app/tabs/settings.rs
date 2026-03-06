use crate::app::{App, hotkeys::validate_hotkey};
use egui::{Color32, Context, CornerRadius, Frame, Stroke, Ui};
use tracing::{error, info};

pub(crate) fn draw(app: &mut App, ui: &mut Ui, ctx: &Context) {
    ui.heading("Настройки");

    ui.add_space(10.0);

    // Группа: Общие настройки
    Frame::group(ui.style())
        .stroke(Stroke::new(1.0, Color32::from_gray(60)))
        .corner_radius(CornerRadius::same(5))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(egui::RichText::new("Общие").strong().size(14.0));
            ui.add_space(5.0);
            let old_aot = app.config.always_on_top;
            let checkbox_response = ui.checkbox(&mut app.config.always_on_top, "Поверх всех окон");
            if checkbox_response.changed() && old_aot != app.config.always_on_top {
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    if app.config.always_on_top {
                        egui::WindowLevel::AlwaysOnTop
                    } else {
                        egui::WindowLevel::Normal
                    },
                ));
                if let Err(e) = app.config.save() {
                    error!("Не удалось сохранить конфигурацию: {}", e);
                }
            }
        });

    ui.add_space(10.0);

    // Группа: Горячие клавиши кликера
    Frame::group(ui.style())
        .stroke(Stroke::new(1.0, Color32::from_gray(60)))
        .corner_radius(CornerRadius::same(5))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new("Горячие клавиши кликера")
                    .strong()
                    .size(14.0),
            );
            ui.add_space(5.0);

            // Клавиша установки позиции
            ui.horizontal(|ui| {
                ui.label("Установить координаты:");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let is_valid = validate_hotkey(&app.temp_set_position);

                    let text_edit = egui::TextEdit::singleline(&mut app.temp_set_position)
                        .desired_width(120.0)
                        .hint_text("Например: F6");

                    let response = if is_valid {
                        ui.add(text_edit)
                    } else {
                        ui.visuals_mut().extreme_bg_color = Color32::from_rgb(80, 20, 20);
                        let resp = ui.add(text_edit);
                        ui.reset_style();
                        resp
                    };

                    if response.changed() {
                        if is_valid && app.temp_set_position != app.config.hotkeys.set_position {
                            app.config.hotkeys.set_position = app.temp_set_position.clone();
                            if let Err(e) = app.hotkeys.update_keys(
                                &app.config.hotkeys.set_position,
                                &app.config.hotkeys.toggle,
                            ) {
                                error!("Не удалось обновить горячие клавиши: {}", e);
                                app.status = format!("Ошибка обновления клавиш: {}", e);
                            } else {
                                info!(
                                    "Клавиша установки позиции изменена на: {}",
                                    app.temp_set_position
                                );
                                if let Err(e) = app.config.save() {
                                    error!("Не удалось сохранить конфигурацию: {}", e);
                                    app.status = format!("Ошибка сохранения конфигурации: {}", e);
                                } else {
                                    app.status =
                                        format!("Клавиша установки: {}", app.temp_set_position);
                                }
                            }
                        } else if !is_valid {
                            app.status =
                                format!("Неверный формат клавиши: {}", app.temp_set_position);
                        }
                    }
                });
            });

            ui.add_space(5.0);

            // Клавиша переключения
            ui.horizontal(|ui| {
                ui.label("Запуск/остановка:");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let is_valid = validate_hotkey(&app.temp_toggle);

                    let text_edit = egui::TextEdit::singleline(&mut app.temp_toggle)
                        .desired_width(120.0)
                        .hint_text("Например: F7");

                    let response = if is_valid {
                        ui.add(text_edit)
                    } else {
                        ui.visuals_mut().extreme_bg_color = Color32::from_rgb(80, 20, 20);
                        let resp = ui.add(text_edit);
                        ui.reset_style();
                        resp
                    };

                    if response.changed() {
                        if is_valid && app.temp_toggle != app.config.hotkeys.toggle {
                            app.config.hotkeys.toggle = app.temp_toggle.clone();
                            if let Err(e) = app.hotkeys.update_keys(
                                &app.config.hotkeys.set_position,
                                &app.config.hotkeys.toggle,
                            ) {
                                error!("Не удалось обновить горячие клавиши: {}", e);
                                app.status = format!("Ошибка обновления клавиш: {}", e);
                            } else {
                                info!("Клавиша переключения изменена на: {}", app.temp_toggle);
                                if let Err(e) = app.config.save() {
                                    error!("Не удалось сохранить конфигурацию: {}", e);
                                    app.status = format!("Ошибка сохранения конфигурации: {}", e);
                                } else {
                                    app.status =
                                        format!("Клавиша переключения: {}", app.temp_toggle);
                                }
                            }
                        } else if !is_valid {
                            app.status = format!("Неверный формат клавиши: {}", app.temp_toggle);
                        }
                    }
                });
            });

            ui.add_space(5.0);
            ui.label(
                egui::RichText::new("Формат: F1-F12, A-Z, 0-9, Ctrl+Key, Shift+Key, Alt+Key")
                    .small()
                    .italics(),
            );
            ui.label(
                egui::RichText::new("Примеры: F6, Ctrl+S, Shift+F1, Alt+A")
                    .small()
                    .italics(),
            );
        });
}

use crate::app::App;
use egui::Ui;
use enigo::Mouse;
use tracing::{error, info, warn};

pub(crate) fn draw(app: &mut App, ui: &mut Ui) {
    ui.heading("Кликер");

    if let Ok((x, y)) = app.enigo.location() {
        ui.horizontal(|ui| {
            ui.label("Установленные:");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.monospace(app.clicker.target());
            });
        });

        ui.horizontal(|ui| {
            let interval_label = ui.label("Интервал:");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let is_valid =
                    app.interval_input.parse::<u64>().is_ok() && !app.interval_input.is_empty();

                let text_edit = egui::TextEdit::singleline(&mut app.interval_input)
                    .desired_width(120.0)
                    .char_limit(6)
                    .hint_text("мс");

                let response = if is_valid {
                    ui.add(text_edit).labelled_by(interval_label.id)
                } else {
                    ui.visuals_mut().extreme_bg_color = egui::Color32::from_rgb(80, 20, 20);
                    let resp = ui.add(text_edit).labelled_by(interval_label.id);
                    ui.reset_style();
                    resp
                };

                if response.changed()
                    && let Ok(value) = app.interval_input.parse::<u64>()
                    && value > 0
                {
                    app.clicker.set_interval(value);
                    app.status = format!("Интервал установлен: {} мс", value);
                    info!("Интервал кликера изменён на {} мс", value);

                    if let Err(e) = app.config.update_and_save(|config| {
                        config.default_interval = value;
                    }) {
                        warn!("Не удалось сохранить интервал в конфигурацию: {}", e);
                    }
                }
            });
        });
        ui.separator();
        ui.label(format!(
            "{} - Установить координаты",
            app.config.hotkeys.set_position
        ));
        ui.label(format!("{} - Старт / Стоп", app.config.hotkeys.toggle));

        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.available_height()),
            egui::Layout::bottom_up(egui::Align::RIGHT),
            |ui| {
                ui.monospace(format!("X: {:4}  Y: {:4}", x, y));
            },
        );
    } else {
        app.status = "Не удаётся получить координаты курсора".to_owned();
        error!("Не удаётся получить координаты курсора");
    }
}

use egui::{Button, Ui};

use crate::app::{App, utils::truncate_str};

pub(crate) fn draw(app: &mut App, ui: &mut Ui) {
    ui.heading("Монитор клавиш");
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        let label = if app.key_monitor.is_active() {
            "⏹ Остановить"
        } else {
            "▶ Запустить"
        };
        if ui
            .add_sized([100.0, ui.spacing().interact_size.y], Button::new(label))
            .clicked()
        {
            app.key_monitor.toggle();
            let monitor_status = if app.key_monitor.is_active() {
                "запущен"
            } else {
                "остановлен"
            };
            app.status = format!("Монитор клавиш {monitor_status}");
        }

        if ui.button("Очистить").clicked() {
            app.key_monitor.clear();
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{} записей", app.key_monitor.entry_count()))
                    .small()
                    .color(egui::Color32::from_gray(140)),
            );
        });
    });

    ui.separator();

    let entries = app.key_monitor.snapshot();

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            egui::Grid::new("monitor_log")
                .num_columns(4)
                .min_col_width(60.0)
                .striped(true)
                .show(ui, |ui| {
                    for entry in &entries {
                        ui.monospace(
                            egui::RichText::new(&entry.time)
                                .small()
                                .color(egui::Color32::from_gray(140)),
                        );
                        ui.monospace(
                            egui::RichText::new(&entry.key)
                                .small()
                                .color(egui::Color32::from_rgb(180, 220, 255)),
                        );
                        ui.label(egui::RichText::new(&entry.process).small());
                        ui.label(
                            egui::RichText::new(truncate_str(&entry.window, 50))
                                .small()
                                .color(egui::Color32::from_gray(180)),
                        );
                        ui.end_row();
                    }
                });
        });
}

use crate::app::App;
use egui::Ui;

pub(crate) fn draw(_app: &mut App, ui: &mut Ui) {
    ui.heading("О программе");
    ui.label("Версия 0.1.0");
}

use std::collections::HashSet;
use std::fs;
use std::io::BufWriter;
use std::path::PathBuf;

use crate::app::App;
use crate::app::IcoImage;
use egui::RichText;
use egui::{Color32, Context, Painter, Rect, Stroke, Ui};
use egui_extras::{Column, TableBuilder};

const ALLOWED_EXT: &[&str] = &["png", "bmp", "jpg", "jpeg"];
const STANDARD_SIZES: &[u32] = &[16, 24, 32, 48, 64, 128, 256];

#[derive(PartialEq)]
enum RowState {
    Ok,
    Oversized,
    DuplicateResolution,
}

fn row_states(images: &[IcoImage]) -> Vec<RowState> {
    let mut states: Vec<RowState> = images
        .iter()
        .map(|img| {
            if img.resolution.width > 256 || img.resolution.height > 256 {
                RowState::Oversized
            } else {
                RowState::Ok
            }
        })
        .collect();

    for i in 0..images.len() {
        for j in (i + 1)..images.len() {
            if images[i].resolution.width == images[j].resolution.width
                && images[i].resolution.height == images[j].resolution.height
            {
                if states[i] == RowState::Ok {
                    states[i] = RowState::DuplicateResolution;
                }
                if states[j] == RowState::Ok {
                    states[j] = RowState::DuplicateResolution;
                }
            }
        }
    }

    states
}

fn best_source_idx(images: &[IcoImage], target: u32) -> Option<usize> {
    if images.is_empty() {
        return None;
    }

    let best_down = images
        .iter()
        .enumerate()
        .filter(|(_, img)| img.resolution.width >= target && img.resolution.height >= target)
        .min_by_key(|(_, img)| img.resolution.width.max(img.resolution.height));

    if let Some((i, _)) = best_down {
        return Some(i);
    }

    images
        .iter()
        .enumerate()
        .max_by_key(|(_, img)| img.resolution.width.max(img.resolution.height))
        .map(|(i, _)| i)
}

pub(crate) fn draw(app: &mut App, ui: &mut Ui, ctx: &Context) {
    ui.heading("Сборка иконок");

    handle_file_drop(app, ctx);

    let painter = ui.painter();
    let rect = ui.max_rect().expand(4.0);

    match hover_state(ctx) {
        HoverState::Valid => {
            draw_dashed_rect(
                painter,
                rect,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(72, 200, 140, 180)),
            );
        }
        HoverState::Invalid => {
            draw_dashed_rect(
                painter,
                rect,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(210, 90, 90, 180)),
            );
        }
        HoverState::None => {}
    }

    let states = row_states(&app.ico_images);
    let has_errors = states.iter().any(|s| *s != RowState::Ok);
    let can_build = !app.ico_images.is_empty() && !has_errors;

    let present_sizes: HashSet<u32> = app
        .ico_images
        .iter()
        .filter(|img| img.resolution.width == img.resolution.height)
        .map(|img| img.resolution.width)
        .collect();

    let mut sorted_extra: Vec<u32> = app
        .ico_extra_sizes
        .iter()
        .copied()
        .filter(|s| !present_sizes.contains(s))
        .collect();
    sorted_extra.sort_unstable();

    let auto_entries: Vec<(u32, Option<usize>)> = sorted_extra
        .iter()
        .map(|&s| (s, best_source_idx(&app.ico_images, s)))
        .collect();

    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        ui.add_space(2.0);

        ui.add_enabled_ui(can_build, |ui| {
            if ui
                .add_sized(
                    [ui.available_width(), 28.0],
                    egui::Button::new("Собрать .ico"),
                )
                .clicked()
            {
                build_ico(&app.ico_images, &sorted_extra);
            }
        });

        if has_errors {
            let has_oversized = states.contains(&RowState::Oversized);
            let has_dupes = states.contains(&RowState::DuplicateResolution);

            let msg = match (has_oversized, has_dupes) {
                (true, true) => "Есть файлы больше 256×256 и дублирующиеся разрешения",
                (true, false) => "Есть файлы больше 256×256",
                (false, true) => "Есть файлы с одинаковым разрешением",
                _ => "",
            };

            ui.add_space(2.0);
            ui.label(
                RichText::new(msg)
                    .size(11.0)
                    .color(Color32::from_rgb(200, 120, 50)),
            );
        }

        if !app.ico_images.is_empty() {
            let available_to_add: Vec<u32> = STANDARD_SIZES
                .iter()
                .copied()
                .filter(|s| !present_sizes.contains(s) && !app.ico_extra_sizes.contains(s))
                .collect();

            if !available_to_add.is_empty() {
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("Добавить авто-размер:")
                            .size(11.0)
                            .color(ui.visuals().weak_text_color()),
                    );
                    for size in available_to_add {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(format!("{}x{}", size, size)).size(11.0),
                                )
                                .small(),
                            )
                            .clicked()
                        {
                            app.ico_extra_sizes.insert(size);
                        }
                    }
                });
            }
        }

        ui.add_space(4.0);

        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            let mut to_remove_real: Option<usize> = None;
            let mut to_remove_extra: Option<u32> = None;
            let default_text_color = ui.visuals().text_color();
            let weak_color = ui.visuals().weak_text_color();
            let auto_bg = Color32::from_rgba_unmultiplied(80, 120, 200, 12);
            let auto_res_color = Color32::from_rgb(100, 150, 210);
            let table_height = ui.available_height();
            TableBuilder::new(ui)
                .min_scrolled_height(0.0)
                .max_scroll_height(table_height)
                .column(Column::remainder())
                .column(Column::exact(70.0))
                .column(Column::exact(70.0))
                .column(Column::exact(20.0))
                .header(12.0, |mut header| {
                    header.col(|ui| {
                        ui.label(RichText::new("Файл").size(12.0));
                    });
                    header.col(|ui| {
                        ui.label(RichText::new("Разрешение").size(12.0));
                    });
                    header.col(|ui| {
                        ui.label(RichText::new("Размер").size(12.0));
                    });
                    header.col(|_ui| {});
                })
                .body(|mut body| {
                    for (i, ico) in app.ico_images.iter().enumerate() {
                        let state = &states[i];
                        let (res_color, row_color) = match state {
                            RowState::Oversized => (
                                Color32::from_rgb(210, 80, 80),
                                Some(Color32::from_rgba_unmultiplied(210, 80, 80, 20)),
                            ),
                            RowState::DuplicateResolution => (
                                Color32::from_rgb(210, 155, 40),
                                Some(Color32::from_rgba_unmultiplied(210, 155, 40, 20)),
                            ),
                            RowState::Ok => (default_text_color, None),
                        };

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                if let Some(bg) = row_color {
                                    ui.painter().rect_filled(ui.max_rect(), 0.0, bg);
                                }
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.label(&ico.file_name);
                                    },
                                );
                            });
                            row.col(|ui| {
                                if let Some(bg) = row_color {
                                    ui.painter().rect_filled(ui.max_rect(), 0.0, bg);
                                }
                                let tooltip = match state {
                                    RowState::Oversized => "Превышает максимальный размер 256x256",
                                    RowState::DuplicateResolution => "Дублирующееся разрешение",
                                    RowState::Ok => "",
                                };
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        let label = ui.label(
                                            RichText::new(ico.resolution.to_string())
                                                .color(res_color),
                                        );
                                        if !tooltip.is_empty() {
                                            label.on_hover_text(tooltip);
                                        }
                                    },
                                );
                            });
                            row.col(|ui| {
                                if let Some(bg) = row_color {
                                    ui.painter().rect_filled(ui.max_rect(), 0.0, bg);
                                }
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.label(ico.formatted_size());
                                    },
                                );
                            });
                            row.col(|ui| {
                                if let Some(bg) = row_color {
                                    ui.painter().rect_filled(ui.max_rect(), 0.0, bg);
                                }
                                ui.with_layout(
                                    egui::Layout::centered_and_justified(
                                        egui::Direction::LeftToRight,
                                    ),
                                    |ui| {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new("🗙")
                                                        .size(10.0)
                                                        .color(Color32::from_rgb(180, 80, 80)),
                                                )
                                                .frame(false),
                                            )
                                            .clicked()
                                        {
                                            to_remove_real = Some(i);
                                        }
                                    },
                                );
                            });
                        });
                    }

                    for (target, src_idx) in &auto_entries {
                        let src_label = match src_idx {
                            Some(i) => {
                                let src = &app.ico_images[*i];
                                format!("<- {}x{}", src.resolution.width, src.resolution.height)
                            }
                            None => "<- ?".to_string(),
                        };
                        let is_upscale = src_idx
                            .map(|i| {
                                let src = &app.ico_images[i];
                                src.resolution.width < *target || src.resolution.height < *target
                            })
                            .unwrap_or(false);

                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.painter().rect_filled(ui.max_rect(), 0.0, auto_bg);
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new("авто")
                                                .size(11.0)
                                                .italics()
                                                .color(weak_color),
                                        );
                                    },
                                );
                            });
                            row.col(|ui| {
                                ui.painter().rect_filled(ui.max_rect(), 0.0, auto_bg);
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(format!("{}x{}", target, target))
                                                .color(auto_res_color),
                                        );
                                    },
                                );
                            });
                            row.col(|ui| {
                                ui.painter().rect_filled(ui.max_rect(), 0.0, auto_bg);
                                let src_color = if is_upscale {
                                    Color32::from_rgb(200, 150, 50)
                                } else {
                                    weak_color
                                };
                                let tooltip = if is_upscale {
                                    "Источник меньше цели — будет апскейл (качество хуже)"
                                } else {
                                    "Источник для масштабирования (Lanczos3)"
                                };
                                ui.with_layout(
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(&src_label).size(11.0).color(src_color),
                                        )
                                        .on_hover_text(tooltip);
                                    },
                                );
                            });
                            row.col(|ui| {
                                ui.painter().rect_filled(ui.max_rect(), 0.0, auto_bg);
                                ui.with_layout(
                                    egui::Layout::centered_and_justified(
                                        egui::Direction::LeftToRight,
                                    ),
                                    |ui| {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new("🗙")
                                                        .size(10.0)
                                                        .color(Color32::from_rgb(180, 80, 80)),
                                                )
                                                .frame(false),
                                            )
                                            .clicked()
                                        {
                                            to_remove_extra = Some(*target);
                                        }
                                    },
                                );
                            });
                        });
                    }
                });

            if let Some(i) = to_remove_real {
                app.ico_images.remove(i);
            }
            if let Some(s) = to_remove_extra {
                app.ico_extra_sizes.remove(&s);
            }
        });
    });
}

enum HoverState {
    None,
    Valid,
    Invalid,
}

fn hover_state(ctx: &egui::Context) -> HoverState {
    let files = ctx.input(|i| i.raw.hovered_files.clone());
    if files.is_empty() {
        return HoverState::None;
    }
    let all_ok = files
        .iter()
        .all(|f| f.path.as_deref().map(is_allowed).unwrap_or(false));

    if all_ok {
        HoverState::Valid
    } else {
        HoverState::Invalid
    }
}

fn handle_file_drop(app: &mut App, ctx: &egui::Context) {
    let existing_paths: HashSet<PathBuf> =
        app.ico_images.iter().map(|img| img.path.clone()).collect();

    for file in ctx.input(|i| i.raw.dropped_files.clone()) {
        if let Some(path) = file.path
            && is_allowed(&path)
            && !existing_paths.contains(&path)
            && let Ok(ico_image) = IcoImage::new(path)
        {
            if ico_image.resolution.width == ico_image.resolution.height {
                app.ico_extra_sizes.remove(&ico_image.resolution.width);
            }
            app.ico_images.push(ico_image);
        }
    }
}

fn is_allowed(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| ALLOWED_EXT.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn draw_dashed_rect(painter: &Painter, rect: Rect, stroke: Stroke) {
    let dash = 6.0;
    let gap = 4.0;

    let sides = [
        [rect.left_top(), rect.right_top()],
        [rect.right_top(), rect.right_bottom()],
        [rect.right_bottom(), rect.left_bottom()],
        [rect.left_bottom(), rect.left_top()],
    ];

    let shapes: Vec<_> = sides
        .iter()
        .flat_map(|pts| egui::Shape::dashed_line(pts, stroke, dash, gap))
        .collect();

    painter.extend(shapes);
}

fn build_ico(images: &[IcoImage], extra_sizes: &[u32]) {
    let Some(dest) = rfd::FileDialog::new()
        .set_file_name("output.ico")
        .add_filter("ICO файл", &["ico"])
        .save_file()
    else {
        return;
    };

    match assemble_ico(images, extra_sizes, &dest) {
        Ok(()) => {
            rfd::MessageDialog::new()
                .set_title("Готово")
                .set_description(format!("Файл сохранён:\n{}", dest.display()))
                .set_level(rfd::MessageLevel::Info)
                .show();
        }
        Err(e) => {
            rfd::MessageDialog::new()
                .set_title("Ошибка")
                .set_description(format!("Не удалось собрать .ico:\n{e}"))
                .set_level(rfd::MessageLevel::Error)
                .show();
        }
    }
}

fn assemble_ico(
    images: &[IcoImage],
    extra_sizes: &[u32],
    dest: &std::path::Path,
) -> Result<(), anyhow::Error> {
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for img in images {
        let rgba = image::open(&img.path)?.into_rgba8();
        let (w, h) = (rgba.width(), rgba.height());
        anyhow::ensure!(
            w <= 256 && h <= 256,
            "Изображение {} превышает максимальный размер 256x256",
            img.file_name
        );
        let icon_image = ico::IconImage::from_rgba_data(w, h, rgba.into_raw());
        icon_dir.add_entry(ico::IconDirEntry::encode(&icon_image)?);
    }

    for &target in extra_sizes {
        let src_idx = best_source_idx(images, target)
            .ok_or_else(|| anyhow::anyhow!("Нет источника для размера {target}x{target}"))?;

        let rgba = image::open(&images[src_idx].path)?.into_rgba8();
        let resized =
            image::imageops::resize(&rgba, target, target, image::imageops::FilterType::Lanczos3);
        let icon_image = ico::IconImage::from_rgba_data(target, target, resized.into_raw());
        icon_dir.add_entry(ico::IconDirEntry::encode(&icon_image)?);
    }

    let file = fs::File::create(dest)?;
    icon_dir.write(BufWriter::new(file))?;

    Ok(())
}

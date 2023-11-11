use bevy_editor_pls_core::widgets::selectable::Selectable;
use bevy_inspector_egui::egui::{Align, Color32, Layout, Ui};
use egui_extras::{Column, TableBuilder};

use super::Logs;

pub fn draw_console_logs(ui: &mut Ui, filter: &mut log::Level, logs: &Logs) {
    let map_logs = logs.get_logs();

    ui.horizontal(|ui| {
        if let Some(s) = Selectable::new(
            &[
                ("trace", log::Level::Trace, None),
                ("info", log::Level::Info, None),
                ("warn", log::Level::Warn, None),
                ("error", log::Level::Error, None),
            ],
            *filter,
            14,
            0,
            Color32::DARK_GRAY,
        )
        .show(ui)
        {
            *filter = s;
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("Clear").clicked() {
                logs.clear();
            }
        });
    });

    ui.separator();

    let table = TableBuilder::new(ui)
        .striped(true)
        .resizable(!map_logs.is_empty())
        .cell_layout(Layout::left_to_right(Align::Center))
        .column(Column::initial(30.0).resizable(false))
        .column(Column::initial(50.0).resizable(false))
        .column(Column::initial(100.0).at_least(40.0).clip(true))
        .column(Column::remainder().clip(true))
        .min_scrolled_height(0.0);

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.strong("");
            });
            header.col(|ui| {
                ui.strong("Level");
            });
            header.col(|ui| {
                ui.strong("File");
            });
            header.col(|ui| {
                ui.strong("Details");
            });
        })
        .body(|mut body| {
            for (log, n) in &map_logs {
                if *filter < log.level_log {
                    continue;
                }
                body.row(30., |mut row| {
                    row.col(|ui| {
                        if n >= &99 {
                            ui.label("+99");
                        } else if n > &0 {
                            ui.label(n.to_string());
                        }
                    });
                    row.col(|ui| {
                        ui.label(log.level_log.to_string());
                    });
                    row.col(|ui| {
                        ui.label(format!("{}:{}", log.file, log.line));
                    });
                    row.col(|ui| {
                        ui.label(&log.details);
                    });
                });
            }
        });
}

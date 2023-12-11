#![allow(dead_code)]
use bevy_inspector_egui::egui::{self, Button, Color32, KeyboardShortcut, Layout, Rounding, Vec2};

#[derive(Clone, PartialEq, Eq)]
pub struct Selectable<T: Clone + PartialEq> {
    tabs: Vec<(String, T, Option<KeyboardShortcut>)>,
    changed: bool,
    font_size: u32,
    space: u32,
    selected_color: Color32,
    selected: Option<T>,
}

impl<T> Selectable<T>
where
    T: Clone + PartialEq,
{
    pub fn new<N: ToString>(
        tabs: &[(N, T, Option<KeyboardShortcut>)],
        default_selected: T,
        font_size: u32,
        space: u32,
        selected_color: Color32,
    ) -> Self {
        Self {
            space,
            font_size,
            selected_color,
            changed: false,
            selected: Some(default_selected),
            tabs: tabs
                .iter()
                .map(|t| (t.0.to_string(), t.1.clone(), t.2.clone()))
                .collect(),
        }
    }

    pub fn add_tab(&mut self, pos: usize, name: &str, tab: T, shortcut: Option<KeyboardShortcut>) {
        self.tabs.insert(pos, (name.to_string(), tab, shortcut));
    }

    pub fn remove_tab(&mut self, pos: usize) {
        self.tabs.remove(pos);
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<T> {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::left_to_right(egui::Align::Center), |ui| {
                ui.style_mut().visuals.button_frame = false;
                for (tab, d, shortcut) in self.tabs.iter() {
                    let item = if self.selected.as_ref().is_some_and(|s| s == d) {
                        Button::new(tab)
                            .fill(self.selected_color)
                            .min_size(Vec2::splat(self.font_size as f32))
                            .rounding(Rounding::ZERO)
                    } else {
                        Button::new(tab)
                            .min_size(Vec2::splat(self.font_size as f32))
                            .rounding(Rounding::ZERO)
                    };
                    let item = ui.add(item);
                    ui.add_space(self.space as f32);
                    if item.clicked()
                        || shortcut.is_some_and(|s| ui.input_mut(|i| i.consume_shortcut(&s)))
                    {
                        self.selected = Some(d.clone());
                    }
                }
            });
        });
        self.selected.clone()
    }
}

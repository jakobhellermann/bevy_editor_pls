use bevy::{pbr::DirectionalLightShadowMap, prelude::*, render::renderer::RenderDevice};
use bevy_editor_pls_core::editor_window::{EditorWindow, EditorWindowContext};
use bevy_inspector_egui::{
    egui::{self, RichText},
    options::NumberAttributes,
    Context, Inspectable,
};

pub struct RendererWindow;

impl EditorWindow for RendererWindow {
    type State = ();
    const NAME: &'static str = "Renderer";
    const DEFAULT_SIZE: (f32, f32) = (480.0, 240.0);

    fn ui(world: &mut World, _: EditorWindowContext, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let render_device = world.get_resource::<RenderDevice>().unwrap();

                let limits = render_device.limits();
                let features = render_device.features();

                ui.heading("Settings");
                egui::Grid::new("directional_light_shadow_map").show(ui, |ui| {
                    let mut directional_light_shadow_map = world
                        .get_resource_mut::<DirectionalLightShadowMap>()
                        .unwrap();
                    ui.label("Directional light shadow map size");
                    let mut size = directional_light_shadow_map.size;
                    if size.ui(
                        ui,
                        NumberAttributes::min(1).with_speed(4.0),
                        &mut Context::new_shared(None),
                    ) {
                        directional_light_shadow_map.size = size;
                    }
                    ui.end_row();
                });

                ui.collapsing("Limits", |ui| {
                    ui.label(RichText::new(format!("{:#?}", limits)).monospace());
                });
                ui.collapsing("Features", |ui| {
                    let features = format!("{:#?}", features);
                    for feature in features.split(" | ") {
                        ui.label(RichText::new(format!("- {}", feature)).monospace());
                    }
                });
            });
    }
}

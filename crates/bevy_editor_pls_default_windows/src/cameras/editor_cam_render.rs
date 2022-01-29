use bevy::{
    core_pipeline::{self, AlphaMask3d, Opaque3d, Transparent3d},
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_graph::{self, RenderGraph, SlotValue},
        render_phase::RenderPhase,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        renderer::RenderDevice,
        texture::{BevyDefault, TextureCache},
        view::{ExtractedView, ExtractedWindows, ViewTarget, VisibleEntities, WindowSystem},
        RenderApp, RenderStage,
    },
};
use bevy_editor_pls_core::{Editor, EditorState};

use super::{ActiveEditorCamera, CameraWindow, EditorCamKind};

pub fn setup(app: &mut App) {
    let render_app = app.sub_app_mut(RenderApp);
    render_app
        .add_system_to_stage(RenderStage::Extract, extract_editor_cameras)
        .add_system_to_stage(
            RenderStage::Prepare,
            prepare_editor_view_targets.after(WindowSystem::Prepare),
        );
    let editor_cam_node = EditorCamDriverNode::new(&mut render_app.world);
    let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
    let id = render_graph.add_node("editor_cam_driver_node", editor_cam_node);
    render_graph
        .add_node_edge(core_pipeline::node::CLEAR_PASS_DRIVER, id)
        .unwrap();
    render_graph
        .add_node_edge(id, core_pipeline::node::MAIN_PASS_DRIVER)
        .unwrap();
}

pub const EDITOR_CAMERA_FLYCAM: &'static str = "editor_camera_flycam";

fn extract_editor_cameras(
    editor: Res<Editor>,
    editor_state: Res<EditorState>,
    mut commands: Commands,
    windows: Res<Windows>,
    query: Query<(Entity, &Camera, &GlobalTransform, &VisibleEntities), With<ActiveEditorCamera>>,
) {
    let camera_window_state = editor.window_state::<CameraWindow>().unwrap();

    if !editor_state.active {
        return;
    }

    match camera_window_state.editor_cam {
        EditorCamKind::D3Free => {}
    }

    for (entity, camera, transform, visible_entities) in query.iter() {
        if let Some(window) = windows.get(camera.window) {
            let mut commands = commands.get_or_spawn(entity);
            commands.insert_bundle((
                ExtractedCamera {
                    window_id: camera.window,
                    name: camera.name.clone(),
                },
                ExtractedView {
                    projection: camera.projection_matrix,
                    transform: *transform,
                    width: window.physical_width().max(1),
                    height: window.physical_height().max(1),
                    near: camera.near,
                    far: camera.far,
                },
                visible_entities.clone(),
                ActiveEditorCamera,
            ));
            commands.insert_bundle((
                RenderPhase::<Opaque3d>::default(),
                RenderPhase::<AlphaMask3d>::default(),
                RenderPhase::<Transparent3d>::default(),
            ));
        }
    }
}

fn prepare_editor_view_targets(
    mut commands: Commands,
    windows: Res<ExtractedWindows>,
    msaa: Res<Msaa>,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
    cameras: Query<(Entity, &ExtractedCamera), With<ActiveEditorCamera>>,
) {
    for (entity, camera) in cameras.iter() {
        let window = match windows.get(&camera.window_id) {
            Some(window) => window,
            None => continue,
        };
        let swap_chain_texture = match &window.swap_chain_texture {
            Some(texture) => texture,
            _ => continue,
        };
        let sampled_target = if msaa.samples > 1 {
            let sampled_texture = texture_cache.get(
                &render_device,
                TextureDescriptor {
                    label: Some("sampled_color_attachment_texture"),
                    size: Extent3d {
                        width: window.physical_width,
                        height: window.physical_height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: msaa.samples,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::bevy_default(),
                    usage: TextureUsages::RENDER_ATTACHMENT,
                },
            );
            Some(sampled_texture.default_view.clone())
        } else {
            None
        };

        commands.entity(entity).insert(ViewTarget {
            view: swap_chain_texture.clone(),
            sampled_target,
        });
    }
}

pub struct EditorCamDriverNode {
    query: QueryState<Entity, With<ActiveEditorCamera>>,
}

impl EditorCamDriverNode {
    pub fn new(render_world: &mut World) -> Self {
        Self {
            query: QueryState::new(render_world),
        }
    }
}
impl render_graph::Node for EditorCamDriverNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }
    fn run(
        &self,
        graph: &mut render_graph::RenderGraphContext,
        _render_context: &mut bevy::render::renderer::RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        for entity in self.query.iter_manual(world) {
            graph.run_sub_graph(
                core_pipeline::draw_3d_graph::NAME,
                vec![SlotValue::Entity(entity)],
            )?;
        }

        Ok(())
    }
}

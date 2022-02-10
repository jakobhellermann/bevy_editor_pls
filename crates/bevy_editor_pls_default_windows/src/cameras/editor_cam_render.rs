use bevy::{
    core_pipeline::{self, AlphaMask3d, Opaque3d, Transparent2d, Transparent3d},
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

use super::{CameraWindow, EditorCamKind, EditorCamera2dPanZoom, EditorCamera3dFree};

pub fn setup(app: &mut App) {
    let render_app = app.sub_app_mut(RenderApp);
    render_app
        .add_system_to_stage(RenderStage::Extract, extract_editor_cameras)
        .add_system_to_stage(
            RenderStage::Prepare,
            prepare_editor_view_targets.after(WindowSystem::Prepare),
        );

    let cam3d_driver_node = Cam3DDriverNode::new(&mut render_app.world);
    let cam2d_driver_node = Cam2DDriverNode::new(&mut render_app.world);

    let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();

    let cam3d_id = render_graph.add_node("cam3d_driver_node", cam3d_driver_node);
    let cam2d_id = render_graph.add_node("cam2d_driver_node", cam2d_driver_node);

    render_graph
        .add_node_edge(core_pipeline::node::CLEAR_PASS_DRIVER, cam3d_id)
        .unwrap();
    render_graph
        .add_node_edge(cam3d_id, core_pipeline::node::MAIN_PASS_DRIVER)
        .unwrap();

    render_graph
        .add_node_edge(core_pipeline::node::CLEAR_PASS_DRIVER, cam2d_id)
        .unwrap();
    render_graph
        .add_node_edge(cam2d_id, core_pipeline::node::MAIN_PASS_DRIVER)
        .unwrap();
}

pub const EDITOR_CAMERA_3D_FLYCAM: &'static str = "editor_camera_3d_flycam";
pub const EDITOR_CAMERA_2D_PAN_ZOOM: &'static str = "editor_camera_2d_pan_zoom";

fn extract_editor_cameras(
    editor: Res<Editor>,
    editor_state: Res<EditorState>,
    mut commands: Commands,
    windows: Res<Windows>,
    query_3d_free: Query<
        (Entity, &Camera, &GlobalTransform, &VisibleEntities),
        With<EditorCamera3dFree>,
    >,
    query_2d_panzoom: Query<
        (Entity, &Camera, &GlobalTransform, &VisibleEntities),
        With<EditorCamera2dPanZoom>,
    >,
) {
    let camera_window_state = editor.window_state::<CameraWindow>().unwrap();

    if !editor_state.active {
        return;
    }

    let (entity, camera, transform, visible_entities) = match camera_window_state.editor_cam {
        EditorCamKind::D3Free => query_3d_free.single(),
        EditorCamKind::D2PanZoom => query_2d_panzoom.single(),
    };

    let window = match windows.get(camera.window) {
        Some(window) => window,
        _ => return,
    };

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
            #[cfg(feature = "viewport")]
            viewport: camera.viewport.clone(),
        },
        visible_entities.clone(),
    ));

    match camera_window_state.editor_cam {
        EditorCamKind::D2PanZoom => {
            commands.insert_bundle((RenderPhase::<Transparent2d>::default(), ActiveCamera2d));
        }
        EditorCamKind::D3Free => {
            commands.insert_bundle((
                RenderPhase::<Opaque3d>::default(),
                RenderPhase::<AlphaMask3d>::default(),
                RenderPhase::<Transparent3d>::default(),
                ActiveCamera3d,
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
    cameras: Query<(Entity, &ExtractedCamera), Or<(With<ActiveCamera2d>, With<ActiveCamera3d>)>>,
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
#[derive(Component)]
struct ActiveCamera3d;

pub struct Cam3DDriverNode {
    query: QueryState<Entity, With<ActiveCamera3d>>,
}

impl Cam3DDriverNode {
    pub fn new(render_world: &mut World) -> Self {
        Self {
            query: QueryState::new(render_world),
        }
    }
}
impl render_graph::Node for Cam3DDriverNode {
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

#[derive(Component)]
struct ActiveCamera2d;

pub struct Cam2DDriverNode {
    query: QueryState<Entity, With<ActiveCamera2d>>,
}

impl Cam2DDriverNode {
    pub fn new(render_world: &mut World) -> Self {
        Self {
            query: QueryState::new(render_world),
        }
    }
}
impl render_graph::Node for Cam2DDriverNode {
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
                core_pipeline::draw_2d_graph::NAME,
                vec![SlotValue::Entity(entity)],
            )?;
        }

        Ok(())
    }
}

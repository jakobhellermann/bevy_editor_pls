use bevy::{
    core_pipeline,
    prelude::*,
    render::{
        camera::ExtractedCamera,
        render_asset::RenderAssets,
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
use bevy::core_pipeline::{core_2d, core_3d};
use bevy::core_pipeline::core_2d::Transparent2d;
use bevy::core_pipeline::core_3d::{AlphaMask3d, Opaque3d, Transparent3d};
use bevy::render::Extract;
use bevy::ui::node::UI_PASS_DRIVER;
use bevy_editor_pls_core::{Editor, EditorState};

use super::{
    CameraWindow, EditorCamKind, EditorCamera2dPanZoom, EditorCamera3dFree, EditorCamera3dPanOrbit,
};

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

    // render_graph
    //     .add_node_edge(UI_PASS_DRIVER, cam3d_id)
    //     .unwrap();
    // render_graph
    //     .add_node_edge(cam3d_id, core_3d::graph::node::MAIN_PASS)
    //     .unwrap();

    // Clear pass removed TODO:
    // render_graph
    //     .add_node_edge(UI_PASS_DRIVER, cam2d_id)
    //     .unwrap();
    // render_graph
    //     .add_node_edge(cam2d_id, core_2d::graph::node::MAIN_PASS)
    //     .unwrap();
}

fn extract_editor_cameras(
    editor: Extract<Res<Editor>>,
    editor_state: Extract<Res<EditorState>>,
    mut commands: Commands,
    windows: Extract<Res<Windows>>,
    images: Extract<Res<Assets<Image>>>,
    query_3d_free: Extract<Query<
        (Entity, &Camera, &GlobalTransform, &VisibleEntities),
        With<EditorCamera3dFree>,
    >>,
    query_3d_panorbit: Extract<Query<
        (Entity, &Camera, &GlobalTransform, &VisibleEntities),
        With<EditorCamera3dPanOrbit>,
    >>,
    query_2d_panzoom: Extract<Query<
        (Entity, &Camera, &GlobalTransform, &VisibleEntities),
        With<EditorCamera2dPanZoom>,
    >>,
) {
    let camera_window_state = editor.window_state::<CameraWindow>().unwrap();

    if !editor_state.active {
        return;
    }

    let (entity, camera, transform, visible_entities) = match camera_window_state.editor_cam {
        EditorCamKind::D2PanZoom => query_2d_panzoom.single(),
        EditorCamKind::D3Free => query_3d_free.single(),
        EditorCamKind::D3PanOrbit => query_3d_panorbit.single(),
    };

    let camera: &Camera = camera; // TODO: Delete

    let view_size = match camera.target.get_render_target_info(&windows, &images) {
        Some(info) => info.physical_size,
        _ => return,
    };

    let mut commands = commands.get_or_spawn(entity);
    commands.insert_bundle((
        ExtractedCamera {
            target: camera.target.clone(),
            physical_viewport_size: camera.physical_viewport_size(),
            physical_target_size: camera.physical_target_size(),
            viewport: camera.viewport.clone(),
            render_graph: Default::default(),
            priority: 0
        },
        ExtractedView {
            projection: camera.projection_matrix(),
            transform: *transform,
            width: view_size.x.max(1),
            height: view_size.y.max(1),
        },
        visible_entities.clone(),
    ));

    match camera_window_state.editor_cam {
        EditorCamKind::D2PanZoom => {
            commands.insert_bundle((RenderPhase::<Transparent2d>::default(), ActiveCamera2d));
        }
        EditorCamKind::D3Free | EditorCamKind::D3PanOrbit => {
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
    images: Res<RenderAssets<Image>>,
    msaa: Res<Msaa>,
    render_device: Res<RenderDevice>,
    mut texture_cache: ResMut<TextureCache>,
    cameras: Query<(Entity, &ExtractedCamera), Or<(With<ActiveCamera2d>, With<ActiveCamera3d>)>>,
) {
    for (entity, camera) in cameras.iter() {
        let camera: &ExtractedCamera = camera;
        let size = match camera.physical_target_size {
            Some(size) => size,
            None => continue,
        };
        let swap_chain_texture = match camera.target.get_texture_view(&windows, &images) {
            Some(texture) => texture,
            _ => continue,
        };
        let sampled_target = if msaa.samples > 1 {
            let sampled_texture = texture_cache.get(
                &render_device,
                TextureDescriptor {
                    label: Some("sampled_color_attachment_texture"),
                    size: Extent3d {
                        width: size.x,
                        height: size.y,
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
                core_pipeline::core_3d::graph::NAME,
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
                core_2d::graph::NAME,
                vec![SlotValue::Entity(entity)],
            )?;
        }

        Ok(())
    }
}

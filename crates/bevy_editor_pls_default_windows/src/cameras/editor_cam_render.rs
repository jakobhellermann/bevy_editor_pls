use bevy::{
    core_pipeline::{self, AlphaMask3d, Opaque3d, Transparent3d},
    prelude::*,
    render::{
        camera::ActiveCameras,
        render_graph::{self, RenderGraph, SlotValue},
        render_phase::RenderPhase,
        RenderApp, RenderStage,
    },
};

pub fn setup(app: &mut App) {
    let render_app = app.sub_app_mut(RenderApp);
    render_app.add_system_to_stage(RenderStage::Extract, extract_editor_camera_phases);
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

#[derive(Component)]
struct Flycam;

fn extract_editor_camera_phases(mut commands: Commands, active_cameras: Res<ActiveCameras>) {
    if let Some(editor_cam) = active_cameras.get(EDITOR_CAMERA_FLYCAM) {
        if let Some(entity) = editor_cam.entity {
            commands
                .get_or_spawn(entity)
                .insert_bundle((
                    RenderPhase::<Opaque3d>::default(),
                    RenderPhase::<AlphaMask3d>::default(),
                    RenderPhase::<Transparent3d>::default(),
                ))
                .insert(Flycam);
        }
    }
}

pub struct EditorCamDriverNode {
    query: QueryState<Entity, With<Flycam>>,
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

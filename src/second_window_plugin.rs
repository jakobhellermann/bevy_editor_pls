use bevy::{
    prelude::*,
    render::{
        camera::{ActiveCameras, Camera},
        pass::*,
        render_graph::{base::MainPass, CameraNode, PassNode, RenderGraph, WindowSwapChainNode, WindowTextureNode},
        texture::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsage},
    },
    window::{CreateWindow, WindowDescriptor, WindowId},
};
use bevy_fly_camera::FlyCamera;
use bevy_inspector_egui::bevy_egui;
use bevy_mod_picking::PickingCameraBundle;
use once_cell::sync::Lazy;

static EDITOR_WINDOW_ID: Lazy<WindowId> = Lazy::new(WindowId::new);

use crate::{EditorPlugin, EditorSettings};

pub struct EditorPluginSecondWindow;

impl Plugin for EditorPluginSecondWindow {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(EditorPlugin)
            .add_state(EditorWindowState::CreateWindow)
            .add_startup_system(crate::setup_default_keybindings.system())
            .add_system_set(SystemSet::on_update(EditorWindowState::CreateWindow).with_system(setup_window.system()))
            .add_system_set(SystemSet::on_update(EditorWindowState::Setup).with_system(setup_second_window.system()));

        let mut editor_settings = app.world_mut().get_resource_mut::<EditorSettings>().unwrap();
        editor_settings.auto_pickable = true;
        editor_settings.fly_camera = true;

        editor_settings.window = *EDITOR_WINDOW_ID;
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum EditorWindowState {
    CreateWindow,
    Setup,
    Done,
}

fn setup_window(mut app_state: ResMut<State<EditorWindowState>>, mut create_window_events: EventWriter<CreateWindow>) {
    // sends out a "CreateWindow" event, which will be received by the windowing backend
    create_window_events.send(CreateWindow {
        id: *EDITOR_WINDOW_ID,
        descriptor: WindowDescriptor {
            width: 800.,
            height: 600.,
            vsync: false,
            title: "second window".to_string(),
            ..Default::default()
        },
    });

    app_state.set(EditorWindowState::Setup).unwrap();
}

mod second_window {
    pub const SWAP_CHAIN: &str = "second_window_swap_chain";
    pub const DEPTH_TEXTURE: &str = "second_window_depth_texture";
    pub const CAMERA_NODE: &str = "secondary_camera";
    pub const CAMERA_NAME: &str = "Editor Camera";
    pub const SAMPLED_COLOR_ATTACHMENT: &str = "second_multi_sampled_color_attachment";
    pub const PASS: &str = "second_window_pass";
}

fn setup_pipeline(render_graph: &mut RenderGraph, active_cameras: &mut ActiveCameras, msaa: &Msaa, window_id: WindowId) {
    // here we setup our render graph to draw our second camera to the new window's swap chain

    // add a swapchain node for our new window
    render_graph.add_node(second_window::SWAP_CHAIN, WindowSwapChainNode::new(window_id));

    // add a new depth texture node for our new window
    render_graph.add_node(
        second_window::DEPTH_TEXTURE,
        WindowTextureNode::new(
            window_id,
            TextureDescriptor {
                format: TextureFormat::Depth32Float,
                usage: TextureUsage::OUTPUT_ATTACHMENT,
                sample_count: msaa.samples,
                ..Default::default()
            },
        ),
    );

    // add a new camera node for our new window
    render_graph.add_system_node(second_window::CAMERA_NODE, CameraNode::new(second_window::CAMERA_NAME));

    // add a new render pass for our new window / camera
    let mut second_window_pass = PassNode::<&MainPass>::new(PassDescriptor {
        color_attachments: vec![msaa.color_attachment(
            TextureAttachment::Input("color_attachment".to_string()),
            TextureAttachment::Input("color_resolve_target".to_string()),
            Operations {
                load: LoadOp::Clear(Color::rgb(0.5, 0.5, 0.8)),
                store: true,
            },
        )],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
            attachment: TextureAttachment::Input("depth".to_string()),
            depth_ops: Some(Operations {
                load: LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        }),
        sample_count: msaa.samples,
    });

    second_window_pass.add_camera(second_window::CAMERA_NAME);
    active_cameras.add(second_window::CAMERA_NAME);

    render_graph.add_node(second_window::PASS, second_window_pass);

    render_graph
        .add_slot_edge(
            second_window::SWAP_CHAIN,
            WindowSwapChainNode::OUT_TEXTURE,
            second_window::PASS,
            if msaa.samples > 1 {
                "color_resolve_target"
            } else {
                "color_attachment"
            },
        )
        .unwrap();

    render_graph
        .add_slot_edge(
            second_window::DEPTH_TEXTURE,
            WindowTextureNode::OUT_TEXTURE,
            second_window::PASS,
            "depth",
        )
        .unwrap();

    render_graph
        .add_node_edge(second_window::CAMERA_NODE, second_window::PASS)
        .unwrap();

    if msaa.samples > 1 {
        render_graph.add_node(
            second_window::SAMPLED_COLOR_ATTACHMENT,
            WindowTextureNode::new(
                window_id,
                TextureDescriptor {
                    size: Extent3d {
                        depth_or_array_layers: 1,
                        width: 1,
                        height: 1,
                    },
                    mip_level_count: 1,
                    sample_count: msaa.samples,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::default(),
                    usage: TextureUsage::OUTPUT_ATTACHMENT,
                },
            ),
        );

        render_graph
            .add_slot_edge(
                second_window::SAMPLED_COLOR_ATTACHMENT,
                WindowSwapChainNode::OUT_TEXTURE,
                second_window::PASS,
                "color_attachment",
            )
            .unwrap();
    }

    bevy_egui::setup_pipeline(
        render_graph,
        msaa,
        bevy_egui::RenderGraphConfig {
            window_id,
            egui_pass: "egui_pass2",
            main_pass: second_window::PASS,
            swap_chain_node: second_window::SWAP_CHAIN,
            depth_texture: second_window::DEPTH_TEXTURE,
            sampled_color_attachment: second_window::SAMPLED_COLOR_ATTACHMENT,
            transform_node: "egui_transform2",
        },
    );
}

fn setup_second_window(
    mut commands: Commands,
    mut app_state: ResMut<State<EditorWindowState>>,
    windows: Res<Windows>,
    mut active_cameras: ResMut<ActiveCameras>,
    mut render_graph: ResMut<RenderGraph>,
    msaa: Res<Msaa>,
) {
    if windows.get(*EDITOR_WINDOW_ID).is_none() {
        return;
    };
    setup_pipeline(&mut render_graph, &mut active_cameras, &msaa, *EDITOR_WINDOW_ID);

    // SETUP SCENE

    // second window camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            camera: Camera {
                name: Some(second_window::CAMERA_NAME.to_string()),
                window: *EDITOR_WINDOW_ID,
                ..Default::default()
            },
            // transform: Transform::from_xyz(6.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
            transform: Transform::from_xyz(0.0, 2.0, 10.0),
            ..Default::default()
        })
        .insert(FlyCamera {
            enabled: true,
            sensitivity: 6.0,
            only_if_mouse_down: Some(MouseButton::Left),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default());

    app_state.set(EditorWindowState::Done).unwrap();
}

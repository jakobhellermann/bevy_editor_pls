use crate::{plugin::EditorState, EditorSettings};
use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiContext, egui, Inspectable};

pub struct EditorExtensionSpawn;
impl Plugin for EditorExtensionSpawn {
    fn build(&self, app: &mut AppBuilder) {
        let mut editor_settings = app.world_mut().get_resource_or_insert_with(EditorSettings::default);
        add_to_editor(&mut editor_settings);

        app.init_resource::<SpawnExtensionState>()
            .add_event::<OpenEditorEvent>()
            .add_system(spawn_ui.system());
    }
}

fn add_to_editor(settings: &mut EditorSettings) {
    settings.add_menu_item("Utils", |ui, world| {
        let mut state = world.get_resource_mut::<SpawnExtensionState>().unwrap();
        if ui.button("Spawn").clicked() {
            state.open = true;
        }
    });
}

#[derive(Default)]
struct SpawnExtensionState {
    open: bool,
    shape: Shape,
}

fn spawn_ui(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    editor_settings: Res<EditorSettings>,
    mut editor_state: ResMut<EditorState>,
    mut extension_state: ResMut<SpawnExtensionState>,
    egui_context: Res<EguiContext>,
) {
    if !extension_state.open {
        return;
    }

    let ctx = match egui_context.try_ctx_for_window(editor_settings.window) {
        Some(ctx) => ctx,
        None => return,
    };

    let mut is_open = true;
    egui::Window::new("Spawn Object").open(&mut is_open).show(ctx, |ui| {
        ui.style_mut().wrap = Some(false);

        let context = bevy_inspector_egui::Context::new_shared(Some(egui_context.ctx()));
        ui.vertical(|ui| {
            extension_state.shape.ui(ui, Default::default(), &context);
        });

        if ui.button("Spawn").clicked() {
            spawn(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut editor_state,
                &extension_state.shape,
            );
        }
    });

    if !is_open {
        extension_state.open = false;
    }
}

fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    editor_state: &mut EditorState,
    shape: &Shape,
) {
    let material = materials.add(Color::WHITE.into());
    let mesh = meshes.add(shape.to_mesh());

    let entity = commands
        .spawn_bundle(PbrBundle {
            material,
            mesh,
            ..Default::default()
        })
        .id();
    editor_state.currently_inspected = Some(entity);
}

struct OpenEditorEvent;

#[derive(Inspectable, Default)]
struct Capsule {
    radius: f32,
    rings: usize,
    depth: f32,
    latitudes: usize,
    longitudes: usize,
}

#[derive(Inspectable)]
enum Shape {
    Cube(shape::Cube),
    Box(shape::Box),
    Sphere(shape::Icosphere),
    Torus(shape::Torus),
    Plane(shape::Plane),
    Quad(shape::Quad),
    Capsule(shape::Capsule),
}
impl Default for Shape {
    fn default() -> Self {
        Shape::Cube(shape::Cube::default())
    }
}

impl Shape {
    fn to_mesh(&self) -> Mesh {
        match *self {
            Shape::Cube(s) => s.into(),
            Shape::Box(s) => s.into(),
            Shape::Sphere(s) => s.into(),
            Shape::Torus(s) => s.into(),
            Shape::Plane(s) => s.into(),
            Shape::Quad(s) => s.into(),
            Shape::Capsule(s) => s.into(),
        }
    }
}

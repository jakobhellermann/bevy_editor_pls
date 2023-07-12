use bevy::{
    prelude::*,
    render::{render_graph::RenderGraph, Render, RenderApp},
};
use bevy_mod_debugdump::{render_graph, schedule_graph};
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[derive(Resource)]
pub struct DotGraphs {
    pub main_schedule: Option<String>,
    pub fixed_update_schedule: Option<String>,
    pub render_main_schedule: Option<String>,
    pub render_extract_schedule: Option<String>,
    pub render_graph: Option<String>,
}

pub fn setup(app: &mut App) {
    let render_app = match app.get_sub_app(RenderApp) {
        Ok(render_app) => render_app,
        Err(_label) => {
            return;
        }
    };
    let render_graph = render_app.world.get_resource::<RenderGraph>().unwrap();

    let schedule_settings = schedule_graph::settings::Settings {
        include_system: Some(Box::new(|system| {
            !system.name().starts_with("bevy_editor_pls")
        })),
        ..Default::default()
    };
    let rendergraph_settings = render_graph::settings::Settings::default();

    let main_schedule = app.get_schedule(Main).map(|schedule| {
        schedule_graph::schedule_graph_dot(schedule, &app.world, &schedule_settings)
    });

    let fixed_update_schedule = app.get_schedule(FixedUpdate).map(|schedule| {
        schedule_graph::schedule_graph_dot(schedule, &app.world, &schedule_settings)
    });

    let render_main_schedule = render_app.get_schedule(Render).map(|schedule| {
        schedule_graph::schedule_graph_dot(schedule, &app.world, &schedule_settings)
    });
    let render_extract_schedule = render_app.get_schedule(ExtractSchedule).map(|schedule| {
        schedule_graph::schedule_graph_dot(schedule, &app.world, &schedule_settings)
    });

    let render_graph = render_graph::render_graph_dot(render_graph, &rendergraph_settings);

    app.insert_resource(DotGraphs {
        main_schedule,
        fixed_update_schedule,
        render_main_schedule,
        render_extract_schedule,
        render_graph: Some(render_graph),
    });
}

pub fn execute_dot(dot: &str, format: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut child = Command::new("dot")
        .arg("-T")
        .arg(format)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child.stdin.as_mut().unwrap().write_all(dot.as_bytes())?;

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr),
        ));
    }

    Ok(output.stdout)
}

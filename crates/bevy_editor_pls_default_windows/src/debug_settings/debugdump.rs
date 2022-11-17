use bevy::{
    prelude::*,
    render::{render_graph::RenderGraph, RenderApp, RenderStage},
};
use bevy_mod_debugdump::{render_graph, schedule_graph};
use std::{
    io::Write,
    process::{Command, Stdio},
};

#[derive(Resource)]
pub struct DotGraphs {
    pub schedule_graph: String,
    pub render_schedule_graph: String,
    pub render_graph: String,
}

pub fn setup(app: &mut App) {
    let actual_runner = std::mem::replace(&mut app.runner, Box::new(|_| {}));

    app.set_runner(move |mut app| {
        app.update();

        let render_app = match app.get_sub_app(RenderApp) {
            Ok(render_app) => render_app,
            Err(_label) => {
                error!("No render app");
                return;
            }
        };
        let render_graph = render_app.world.get_resource::<RenderGraph>().unwrap();

        let schedule_style = schedule_graph::ScheduleGraphStyle {
            system_filter: Some(Box::new(|system| {
                !system.name.starts_with("bevy_editor_pls")
            })),
            ..Default::default()
        };
        let rendergraph_style = render_graph::RenderGraphStyle::default();

        let schedule_graph = schedule_graph::schedule_graph_dot_styled(&app, &schedule_style);
        let render_graph =
            render_graph::render_graph_dot_styled(&*render_graph, &rendergraph_style);
        let render_schedule_graph = schedule_graph::schedule_graph_dot_sub_app_styled(
            &app,
            RenderApp,
            &[&RenderStage::Extract],
            &schedule_style,
        );

        app.insert_resource(DotGraphs {
            schedule_graph,
            render_schedule_graph,
            render_graph,
        });

        actual_runner(app);
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

use anyhow::anyhow;
use anyhow::Result;
use bytemuck::NoUninit;
use notan::egui::{self, *};
use notan::prelude::*;
use std::f32::consts::TAU;

#[derive(AppState)]
struct State {
    clear_options: ClearOptions,
    pipeline: Pipeline,
    must_reload_shaders: bool,
    frame_idx: usize,
    constant_buffer: Buffer,
    cb_data: CB,
}

#[repr(C)]
#[derive(Default, Copy, Clone, NoUninit)]
struct CB {
    ar: f32,
    zoom: f32,
    point_count: u32,
    line_thickness: f32,

    radius0: f32,
    initial_phase0: f32,
    cycle_count0: f32,
    fractional_cycles0: f32,
    initial_amplitude0: f32,
    amplitude_decay0: f32,
    rotation0: f32,

    radius1: f32,
    initial_phase1: f32,
    cycle_count1: f32,
    fractional_cycles1: f32,
    initial_amplitude1: f32,
    amplitude_decay1: f32,
    rotation1: f32,
}

impl CB {
    fn as_float_slice(&self) -> &[f32] {
        bytemuck::cast_slice(bytemuck::bytes_of(self))
    }
}

#[notan_main]
fn main() -> Result<(), String> {
    notan::init_with(setup)
        .add_config(EguiConfig)
        .add_config(
            WindowConfig::new()
                .vsync()
                .lazy_loop()
                .resizable()
                .multisampling(8),
        )
        .update(update)
        .draw(draw)
        .build()
}

fn load_pipeline(gfx: &mut Graphics) -> Result<Pipeline> {
    let pipeline = gfx
        .create_pipeline()
        .from_raw(
            &std::fs::read("src/shader.vert")?,
            &std::fs::read("src/shader.frag")?,
        )
        .build()
        .map_err(|e| anyhow!("Error: {}", e))?;
    Ok(pipeline)
}

fn setup(gfx: &mut Graphics) -> State {
    let clear_options = ClearOptions::color(Color::new(0.1, 0.2, 0.3, 1.0));

    let pipeline = match load_pipeline(gfx) {
        Ok(pipeline) => pipeline,
        Err(err) => {
            eprintln!("Error compiling shaders: {}", err);
            panic!();
        }
    };

    let cb: CB = Default::default();

    let constant_buffer = gfx
        .create_uniform_buffer(0, "CB")
        .with_data(cb.as_float_slice())
        .build()
        .unwrap();

    let cb_data = CB {
        ar: 1.0,
        zoom: 1.0,
        point_count: 200_000,
        line_thickness: 0.001,

        radius0: 0.4,
        initial_phase0: 1.5,
        cycle_count0: 22.0,
        fractional_cycles0: 0.01,
        initial_amplitude0: 2.5,
        amplitude_decay0: 0.99,
        rotation0: 0.0,

        radius1: 0.3,
        initial_phase1: 0.5,
        cycle_count1: 20.0,
        fractional_cycles1: 0.01,
        initial_amplitude1: 1.3,
        amplitude_decay1: 0.99,
        rotation1: TAU * 0.25,
    };

    State {
        clear_options,
        pipeline,
        must_reload_shaders: false,
        frame_idx: 0,
        constant_buffer,
        cb_data,
    }
}

fn update(app: &mut App, state: &mut State) {
    if app.keyboard.was_pressed(KeyCode::Escape) {
        app.exit();
    }
    if state.frame_idx % 60 == 0 {
        state.must_reload_shaders = true;
    }
    state.frame_idx += 1;
}

fn draw(gfx: &mut Graphics, plugins: &mut Plugins, state: &mut State) {
    if state.must_reload_shaders {
        state.must_reload_shaders = false;

        match load_pipeline(gfx) {
            Ok(pipeline) => {
                println!("Shaders reloaded");
                state.pipeline = pipeline;
            }
            Err(err) => {
                eprintln!("Error compiling shaders: {}", err);
            }
        };
    }

    let mut renderer = gfx.create_renderer();

    state.cb_data.ar = gfx.size().0 as f32 / gfx.size().1 as f32;

    gfx.set_buffer_data(&state.constant_buffer, state.cb_data.as_float_slice());

    renderer.begin(Some(&state.clear_options));
    renderer.set_pipeline(&state.pipeline);
    renderer.set_primitive(DrawPrimitive::TriangleStrip);
    renderer.draw(0, state.cb_data.point_count as i32);
    renderer.end();

    let output = plugins.egui(|ctx| {
        egui::SidePanel::left("side_panel").show(&ctx, |ui| {

            ui.heading("Penduolum");

            ui.label("Point count");
            ui.add(egui::Slider::new(
                &mut state.cb_data.point_count,
                1000..=1_000_000,
            ));

            ui.label("Line thickness");
            ui.add(egui::Slider::new(
                &mut state.cb_data.line_thickness,
                0.0005..=0.01,
            ));

            ui.label("Zoom");
            ui.add(egui::Slider::new(&mut state.cb_data.zoom, 0.05..=10.0));

            ui.add_space(20.0);

            ui.label("Radius0");
            ui.add(egui::Slider::new(&mut state.cb_data.radius0, 0.0..=0.7));

            ui.label("InitialPhase0");
            ui.add(egui::Slider::new(&mut state.cb_data.initial_phase0, 0.0..=TAU));

            ui.label("CycleCount0");
            ui.add(egui::Slider::new(&mut state.cb_data.cycle_count0, 0.0..=100.0));

            ui.label("FractionalCycles0");
            ui.add(egui::Slider::new(&mut state.cb_data.fractional_cycles0, 0.0..=1.0));

            ui.label("InitialAmplitude0");
            ui.add(egui::Slider::new(&mut state.cb_data.initial_amplitude0, 0.0..=TAU));

            ui.label("AplitudeDecay0");
            ui.add(egui::Slider::new(&mut state.cb_data.amplitude_decay0, 0.5..=1.0));

            ui.label("Rotation0");
            ui.add(egui::Slider::new(&mut state.cb_data.rotation0, 0.0..=TAU));

            ui.add_space(20.0);

            ui.label("Radius1");
            ui.add(egui::Slider::new(&mut state.cb_data.radius1, 0.0..=0.7));
            ui.label("InitialPhase1");
            ui.add(egui::Slider::new(&mut state.cb_data.initial_phase1, 0.0..=TAU));

            ui.label("CycleCount1");
            ui.add(egui::Slider::new(&mut state.cb_data.cycle_count1, 0.0..=100.0));

            ui.label("FractionalCycles1");
            ui.add(egui::Slider::new(&mut state.cb_data.fractional_cycles1, 0.0..=1.0));

            ui.label("InitialAmplitude1");
            ui.add(egui::Slider::new(&mut state.cb_data.initial_amplitude1, 0.0..=TAU));

            ui.label("AplitudeDecay1");
            ui.add(egui::Slider::new(&mut state.cb_data.amplitude_decay1, 0.5..=1.0));

            ui.label("Rotation1");
            ui.add(egui::Slider::new(&mut state.cb_data.rotation1, 0.0..=TAU));

            ui.separator();
        });
    });
    gfx.render(&renderer);
    gfx.render(&output);
}

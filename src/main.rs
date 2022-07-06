use anyhow::Result;
use anyhow::anyhow;
use bytemuck::NoUninit;
use notan::egui::{self, *};
use notan::prelude::*;

#[derive(AppState)]
struct State {
    clear_options: ClearOptions,
    pipeline: Pipeline,
    must_reload_shaders: bool,
    frame_idx: usize,
    constant_buffer: Buffer,
    cb_data : CB,
}


#[repr(C)]
#[derive(Default, Copy, Clone, NoUninit)]
struct CB {
    point_count: u32,
    line_thickness: f32,

	radius0: f32,
 	initial_phase0: f32,
 	period0: f32,
 	initial_amplitude0: f32,
 	amplitude_decay0: f32,

	radius1: f32,
 	initial_phase1: f32,
 	period1: f32,
 	initial_amplitude1: f32,
 	amplitude_decay1: f32,
}

impl CB{
    fn as_float_slice(&self) -> &[f32] {
        bytemuck::cast_slice(bytemuck::bytes_of(self))
    }
}

#[notan_main]
fn main() -> Result<(), String> {
    notan::init_with(setup)
        .add_config(EguiConfig)
        .add_config(WindowConfig::new()
            .vsync()
            .lazy_loop()
            .resizable()
            .multisampling(8))
        .update(update)
        .draw(draw).build()
}

fn load_pipeline(gfx: &mut Graphics) -> Result<Pipeline> {
    let pipeline = gfx
        .create_pipeline()
        .from_raw(
            &std::fs::read("src/shader.vert")?,
            &std::fs::read("src/shader.frag")?)
        .build().map_err(|e| anyhow!("Error: {}",e))?;
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


    let cb_data = CB{
        point_count: 30000,
        line_thickness: 0.001,

	    radius0: 0.4,
 	    initial_phase0: 1.5,
 	    period0: 155.0,
 	    initial_amplitude0: 2.5,
 	    amplitude_decay0: 0.99,

	    radius1: 0.5,
 	    initial_phase1: 0.5,
 	    period1: 85.0,
 	    initial_amplitude1: 1.3,
 	    amplitude_decay1: 0.99,
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
    state.frame_idx +=1;
}

fn draw(gfx: &mut Graphics, plugins: &mut Plugins, state: &mut State) {

    if state.must_reload_shaders {
        state.must_reload_shaders = false;

        match load_pipeline(gfx) {
            Ok(pipeline) => {
                println!("Shaders reloaded");
                state.pipeline = pipeline;
            },
            Err(err) => {
                eprintln!("Error compiling shaders: {}", err);
            }
        };
    }


    let mut renderer = gfx.create_renderer();


    gfx.set_buffer_data( &state.constant_buffer, state.cb_data.as_float_slice());

    renderer.begin(Some(&state.clear_options));
    renderer.set_pipeline(&state.pipeline);
    renderer.set_primitive(DrawPrimitive::TriangleStrip);
    renderer.draw(0, state.cb_data.point_count as i32);
    renderer.end();


    let mut output = plugins.egui(|ctx| {
        egui::SidePanel::left("side_panel").show(&ctx, |ui| {
            use std::f32::consts::TAU;

            ui.heading("Penduolum");
            
            ui.label("Point count");
            ui.add(egui::Slider::new(&mut state.cb_data.point_count, 1000..=1_000_000));
            
            ui.label("Line thickness");
            ui.add(egui::Slider::new(&mut state.cb_data.line_thickness, 0.0005..=0.01).suffix("°"));



            ui.label("Radius0");
            ui.add(egui::Slider::new(&mut state.cb_data.radius0, 0.0..=0.7).suffix("°"));

            ui.label("InitialPhase0");
            ui.add(egui::Slider::new(&mut state.cb_data.initial_phase0, 0.0..=TAU).suffix("°"));
            
            ui.label("Period0");
            ui.add(egui::Slider::new(&mut state.cb_data.period0, 0.0..=200.0).suffix("°"));
           
            ui.label("InitialAmplitude0");
            ui.add(egui::Slider::new(&mut state.cb_data.initial_amplitude0, 0.0..=TAU).suffix("°"));
           
            ui.label("AplitudeDecay0");
            ui.add(egui::Slider::new(&mut state.cb_data.amplitude_decay0, 0.9..=1.0).suffix("°"));
           

            ui.label("Radius1");
            ui.add(egui::Slider::new(&mut state.cb_data.radius1, 0.0..=0.7).suffix("°"));
            ui.label("InitialPhase1");
            ui.add(egui::Slider::new(&mut state.cb_data.initial_phase1, 0.0..=TAU).suffix("°"));
            
            ui.label("Period1");
            ui.add(egui::Slider::new(&mut state.cb_data.period1, 0.0..=200.0).suffix("°"));
           
            ui.label("InitialAmplitude1");
            ui.add(egui::Slider::new(&mut state.cb_data.initial_amplitude1, 0.0..=TAU).suffix("°"));
           
            ui.label("AplitudeDecay1");
            ui.add(egui::Slider::new(&mut state.cb_data.amplitude_decay1, 0.9..=1.0).suffix("°"));
           
           
           
            ui.separator();
        });
    });
    gfx.render(&renderer);
    gfx.render(&output);
}
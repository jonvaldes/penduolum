use anyhow::anyhow;
use anyhow::Result;
use notan::egui::{self, *};
use notan::prelude::*;
use std::f32::consts::{PI, TAU};

struct RangedValue {
    name: String,
    value: f32,
    min: f32,
    max: f32,
    visible: bool,
    animated: bool,
    animation_func: Option<fn(t: f32) -> f32>,
    needs_separator: bool,
}

impl RangedValue {
    pub fn new(name: &str, min: f32, max: f32) -> Self {
        Self {
            name: String::from(name),
            value: (max + min) * 0.5,
            min,
            max,
            animated: false,
            visible: true,
            needs_separator: false,
            animation_func: None,
        }
    }
    pub fn with_default(mut self, default: f32) -> Self {
        self.value = default;
        self
    }
    pub fn invisible(mut self) -> Self {
        self.visible = false;
        self
    }
    pub fn separator(mut self) -> Self {
        self.needs_separator = true;
        self
    }
    pub fn anim_fn(mut self, func: fn(t: f32) -> f32) -> Self {
        self.animation_func = Some(func);
        self
    }

    pub fn animated(mut self) -> Self {
        self.animated = true;
        self
    }
}

#[derive(AppState)]
struct State {
    clear_options: ClearOptions,
    pipeline: Pipeline,
    must_reload_shaders: bool,
    frame_idx: usize,
    constant_buffer: Buffer,
    settings: Vec<RangedValue>,
}

#[notan_main]
fn main() -> Result<(), String> {
    notan::init_with(setup)
        .add_config(EguiConfig)
        .add_config(
            WindowConfig::new()
                .vsync()
                .title("Penduolum")
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

    let constant_buffer = gfx
        .create_uniform_buffer(0, "CB")
        .with_data(&[])
        .build()
        .unwrap();

    let settings = vec![
        RangedValue::new("ar", 0.0, 0.0).invisible(),
        RangedValue::new("Point Count", 3001.0, 1_000_000.0).with_default(200_000.0),
        RangedValue::new("Zoom", 0.1, 3.0).with_default(1.0),
        RangedValue::new("Line Thickness", 0.0005, 0.01)
            .with_default(0.0007)
            .separator(),
        RangedValue::new("Radius 0", 0.0, 1.0).anim_fn(|t| 0.5 + 0.3 * t.sin()),
        RangedValue::new("Initial Phase 0", 0.0, TAU)
            .anim_fn(|t| (t * 0.3).rem_euclid(TAU))
            .animated(),
        RangedValue::new("Cycle Count 0", 0.0, 100.0),
        RangedValue::new("Fractional Cycles 0", 0.0, 1.0),
        RangedValue::new("Initial Amplitude 0", 0.0, TAU)
            .anim_fn(|t| TAU * (0.5 + 0.5 * (t * 0.2).sin())),
        RangedValue::new("Amplitude Decay 0", 0.5, 1.0).with_default(0.97),
        RangedValue::new("Rotation 0", 0.0, TAU)
            .anim_fn(|t| (t * 0.161592).rem_euclid(TAU))
            .animated(),
        RangedValue::new("Radius 1", 0.0, 1.0)
            .with_default(0.3)
            .anim_fn(|t| 0.5 + 0.3 * (t + PI).sin()),
        RangedValue::new("Initial Phase 1", 0.0, TAU)
            .anim_fn(|t| (t * 0.3).rem_euclid(TAU))
            .animated(),
        RangedValue::new("Cycle Count 1", 0.0, 100.0).with_default(20.0),
        RangedValue::new("Fractional Cycles 1", 0.0, 1.0),
        RangedValue::new("Initial Amplitude 1", 0.0, TAU)
            .anim_fn(|t| TAU * (0.5 + 0.3 * (t * 0.2 + PI * 0.5).sin())),
        RangedValue::new("Amplitude Decay 1", 0.5, 1.0).with_default(0.97),
        RangedValue::new("Rotation 1", 0.0, TAU)
            .anim_fn(|t| (t * -0.161592).rem_euclid(TAU))
            .animated(),
    ];

    State {
        clear_options,
        pipeline,
        must_reload_shaders: false,
        frame_idx: 0,
        constant_buffer,
        settings,
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

fn draw(app: &mut App, gfx: &mut Graphics, plugins: &mut Plugins, state: &mut State) {
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

    let t = state.frame_idx as f32 / 60.0;

    let mut renderer = gfx.create_renderer();

    // Set Aspect ratio
    state.settings[0].value = gfx.size().0 as f32 / gfx.size().1 as f32;

    let mut any_animated = false;

    for s in &mut state.settings {
        if s.animated && s.animation_func.is_some() {
            s.value = s.animation_func.unwrap()(t);
            any_animated = true;
        }
    }

    if any_animated {
        app.window().request_frame();
    }

    let settings_floats: Vec<f32> = state.settings.iter().map(|s| s.value).collect();

    gfx.set_buffer_data(&state.constant_buffer, &settings_floats);

    renderer.begin(Some(&state.clear_options));
    renderer.set_pipeline(&state.pipeline);
    renderer.set_primitive(DrawPrimitive::TriangleStrip);
    renderer.draw(0, state.settings[1].value as i32);
    renderer.end();

    let output = plugins.egui(|ctx| {
        egui::SidePanel::left("side_panel").show(&ctx, |ui| {
            ui.heading("Penduolum");

            for s in &mut state.settings {
                if !s.visible {
                    continue;
                }
                ui.label(&s.name);

                ui.horizontal(|ui| {
                    let label_text = if s.animation_func.is_some() { "A" } else { " " };
                    let selectable_label = egui::SelectableLabel::new(s.animated, label_text);
                    let response = ui.add_enabled(s.animation_func.is_some(), selectable_label);
                    if response.clicked() {
                        s.animated = !s.animated;
                    }

                    ui.add(egui::Slider::new(&mut s.value, s.min..=s.max));
                });

                if s.needs_separator {
                    ui.separator();
                }
            }
        });
    });
    gfx.render(&renderer);
    gfx.render(&output);
}

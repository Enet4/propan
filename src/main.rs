extern crate clap;
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_graphics;
extern crate glutin_window;
extern crate graphics;
extern crate itertools;
extern crate nalgebra as na;
extern crate piston;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod camera;
mod controller;
mod editor;
mod game;
mod level;
mod physics;
mod resource;
mod title;
mod util;

use clap::{App, Arg, SubCommand};
use gfx::format::DepthStencil;
use gfx::format::{Formatted, Srgba8};
use gfx::handle::{DepthStencilView, RenderTargetView};
use gfx::memory::Typed;
use gfx::pso::{PipelineData, PipelineState};
use gfx::texture::{FilterMethod, SamplerInfo, WrapMode};
use gfx::traits::*;
use gfx::{CommandBuffer, Device, Resources, Slice};
use gfx_graphics::{Filter, Gfx2d, GlyphCache, TextureSettings};
use glutin_window::{GlutinWindow, OpenGL};
use graphics::character::CharacterCache;
use graphics::Viewport;
use piston::event_loop::*;
use piston::input::*;
use piston::window::{OpenGLWindow, Window, WindowSettings};
use std::path::Path;

use controller::{Controller, ControllerAction, LevelId};
use editor::LevelEditorController;
use game::GameController;
use level::GameLevel;
use resource::{AudioManager, ResourceManage, ResourceManager, SpriteManage, SpriteManager};
use title::TitleController;

type ColorFormat = Srgba8;
type DepthFormat = gfx::format::DepthStencil;

pub const WIDTH: u16 = 320;
pub const HEIGHT: u16 = 200;
pub const DEFAULT_PHYSICAL_WIDTH: u16 = 640;
pub const DEFAULT_PHYSICAL_HEIGHT: u16 = 400;
//pub const PIXEL_SCALE: f32 = DEFAULT_PHYSICAL_WIDTH as f32 / WIDTH as f32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    Title,
    Game(LevelId),
    Editor(Option<String>),
    Exit,
}

fn create_main_targets(
    dim: gfx::texture::Dimensions,
) -> (
    gfx::handle::RenderTargetView<gfx_device_gl::Resources, gfx::format::Srgba8>,
    gfx::handle::DepthStencilView<gfx_device_gl::Resources, gfx::format::DepthStencil>,
) {
    let color_format = <Srgba8 as Formatted>::get_format();
    let depth_format = <DepthFormat as Formatted>::get_format();
    let (output_color, output_stencil) =
        gfx_device_gl::create_main_targets_raw(dim, color_format.0, depth_format.0);
    let output_color: RenderTargetView<_, _> = Typed::new(output_color);
    let output_stencil: DepthStencilView<_, _> = Typed::new(output_stencil);
    (output_color, output_stencil)
}

fn create_physical_viewport(window: &GlutinWindow) -> Viewport {
    let piston::window::Size { width, height } = window.size();
    // physical viewport
    Viewport {
        rect: [0, 0, width as i32, height as i32],
        draw_size: [width as u32, height as u32],
        window_size: [width as u32, height as u32],
    }
}

fn create_viewports(window: &GlutinWindow) -> (Viewport, Viewport) {
    (
        // logical viewport
        Viewport {
            rect: [0, 0, ::WIDTH as i32, ::HEIGHT as i32],
            draw_size: [WIDTH as u32, HEIGHT as u32],
            window_size: [WIDTH as u32, HEIGHT as u32],
        },
        create_physical_viewport(window),
    )
}

fn main() {
    let args = App::new("Propan")
        .subcommand(
            SubCommand::with_name("editor")
                .help("Run the level editor")
                .arg(
                    Arg::with_name("FILE")
                        .index(1)
                        .help("The level file to load")
                        .required(false),
                ),
        ).get_matches();
    let boot = if let Some(args) = args.subcommand_matches("editor") {
        ControllerAction::OpenEditor(args.value_of("FILE").map(String::from))
    } else {
        ControllerAction::LoadTitleScreen
    };

    // configure window
    let opengl = OpenGL::V3_2;

    let phys_width: u32 = DEFAULT_PHYSICAL_WIDTH as u32;
    let phys_height: u32 = DEFAULT_PHYSICAL_HEIGHT as u32;
    let samples = 0;
    let mut window: GlutinWindow = WindowSettings::new("propan", [phys_width, phys_height])
        .srgb(false)
        .vsync(true)
        .resizable(false)
        .opengl(opengl)
        .samples(samples)
        .exit_on_esc(false)
        .build()
        .unwrap();

    let (mut device, mut factory) =
        gfx_device_gl::create(|s| window.get_proc_address(s) as *const std::os::raw::c_void);

    // configure graphics
    let mut g2d = Gfx2d::new(opengl, &mut factory);

    // Create the main color/depth targets.
    let draw_size = window.draw_size();
    let aa = samples as gfx::texture::NumSamples;
    let dim = (
        draw_size.width as u16,
        draw_size.height as u16,
        1,
        aa.into(),
    );
    let (output_color, output_stencil) = create_main_targets(dim);

    let (_lowres_texture, lowres_resource_view, lowres_color) =
        factory.create_render_target(WIDTH, HEIGHT).unwrap();
    let lowres_stencil = factory
        .create_depth_stencil_view_only(WIDTH, HEIGHT)
        .unwrap();
    let mut encoder = factory.create_command_buffer().into();
    let pso = factory
        .create_pipeline_simple(
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/main.glslv")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/shaders/main.glslf")),
            pipe::new(),
        ).unwrap();
    let (video_rect, indices) = get_video_rect();
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&video_rect, indices);
    let sampler = factory.create_sampler(SamplerInfo {
        filter: FilterMethod::Scale, // yay, pixelated!
        wrap_mode: (WrapMode::Border, WrapMode::Border, WrapMode::Border),
        lod_bias: (0.0).into(),
        lod_range: ((0.0).into(), (0.0).into()),
        comparison: None,
        border: [0.0, 0.0, 0.0, 1.0].into(), // black border
    });
    let data = pipe::Data {
        vbuf: vertex_buffer,
        orig: (lowres_resource_view, sampler),
        out: output_color.clone(),
    };

    // create initial viewports: logical viewport never changes,
    // but the physical viewport may change on a window resize
    let (logical_viewport, physical_viewport) = create_viewports(&window);

    // character cache
    let mut cache = GlyphCache::new(
        Path::new("assets/fonts/Monospace.ttf"),
        factory.clone(),
        TextureSettings::new().filter(Filter::Nearest),
    ).unwrap();

    // game resources stuff
    let resource_manager = ResourceManager::new(
        SpriteManager::new(factory).unwrap(),
        AudioManager::new(()).unwrap(),
    );

    // event loop
    let mut events = Events::new(EventSettings::new().swap_buffers(true).max_fps(60).ups(120));
    let mut state = match boot {
        ControllerAction::OpenEditor(p) => GameState::Editor(p),
        _ => GameState::Title,
    };
    // The root loop dispatches a particular controller and runs the game loop in each one.
    loop {
        match state {
            GameState::Title => {
                // initialize title logic stuff
                let mut title = TitleController::new(&resource_manager).unwrap();
                // title loop
                state = run_controller(
                    &mut title,
                    &resource_manager,
                    &mut events,
                    &mut window,
                    &mut device,
                    &mut encoder,
                    &slice,
                    &pso,
                    &data,
                    &lowres_color,
                    &lowres_stencil,
                    &output_color,
                    &output_stencil,
                    logical_viewport,
                    physical_viewport,
                    &mut cache,
                    &mut g2d,
                );
                title.exit();
            }
            GameState::Game(id) => {
                // game logic stuff
                let level = GameLevel::load_by_index("levels/", id).unwrap();
                let mut game = GameController::new(level, &resource_manager).unwrap();

                state = run_controller(
                    &mut game,
                    &resource_manager,
                    &mut events,
                    &mut window,
                    &mut device,
                    &mut encoder,
                    &slice,
                    &pso,
                    &data,
                    &lowres_color,
                    &lowres_stencil,
                    &output_color,
                    &output_stencil,
                    logical_viewport,
                    physical_viewport,
                    &mut cache,
                    &mut g2d,
                );
                game.exit();
            }
            GameState::Editor(path) => {
                // level editor stuff
                let mut editor = if let Some(path) = path {
                    LevelEditorController::load(path, &resource_manager).unwrap()
                } else {
                    LevelEditorController::new(&resource_manager).unwrap()
                };
                state = run_controller(
                    &mut editor,
                    &resource_manager,
                    &mut events,
                    &mut window,
                    &mut device,
                    &mut encoder,
                    &slice,
                    &pso,
                    &data,
                    &lowres_color,
                    &lowres_stencil,
                    &output_color,
                    &output_stencil,
                    logical_viewport,
                    physical_viewport,
                    &mut cache,
                    &mut g2d,
                );
                editor.exit();
            }
            GameState::Exit => {
                return;
            }
        }
    }
}

#[inline]
fn run_controller<C, M, D, R, PD, CB, CC, PM>(
    game: &mut C,
    _resource_manager: M,
    events: &mut Events,
    window: &mut GlutinWindow,
    device: &mut D,
    encoder: &mut gfx::Encoder<R, CB>,
    slice: &Slice<R>,
    lowres_pso: &PipelineState<R, PM>,
    lowres_data: &PD,
    lowres_color: &RenderTargetView<R, Srgba8>,
    lowres_stencil: &DepthStencilView<R, DepthStencil>,
    output_color: &RenderTargetView<R, Srgba8>,
    output_stencil: &DepthStencilView<R, DepthStencil>,
    logical_viewport: Viewport,
    mut physical_viewport: Viewport,
    cache: &mut CC,
    g2d: &mut Gfx2d<R>,
) -> GameState
where
    C: Controller<Res = M>,
    D: Device<CommandBuffer = CB, Resources = R>,
    M: ResourceManage,
    <M as ResourceManage>::Sprite: SpriteManage<Texture = gfx_graphics::Texture<R>>,
    R: Resources,
    PD: PipelineData<R, Meta = PM>,
    CB: CommandBuffer<R>,
    CC: CharacterCache<Texture = gfx_graphics::Texture<R>>,
{
    let mut pixel_scale_w = physical_viewport.window_size[0] as f64 / WIDTH as f64;
    let mut pixel_scale_h = physical_viewport.window_size[1] as f64 / HEIGHT as f64;

    // game loop
    while let Some(e) = events.next(window) {
        // handle window closure
        if e.close_args().is_some() {
            return GameState::Exit;
        }

        // handle window resize
        if e.resize_args().is_some() {
            // reset physical viewport
            physical_viewport = create_physical_viewport(&window);
            pixel_scale_w = physical_viewport.window_size[0] as f64 / WIDTH as f64;
            pixel_scale_h = physical_viewport.window_size[1] as f64 / HEIGHT as f64;
        }

        let a = game.event(&e);
        match a {
            Some(ControllerAction::Exit) => {
                return GameState::Exit;
            }
            Some(ControllerAction::LoadTitleScreen) => {
                return GameState::Title;
            }
            Some(ControllerAction::LoadGame(id)) => {
                return GameState::Game(id);
            }
            Some(ControllerAction::OpenEditor(p)) => {
                return GameState::Editor(p);
            }
            _ => {}
        }

        if let Some(u) = e.update_args() {
            let a = game.update(u);
            match a {
                Some(ControllerAction::Exit) => {
                    return GameState::Exit;
                }
                Some(ControllerAction::LoadTitleScreen) => {
                    return GameState::Title;
                }
                Some(ControllerAction::LoadGame(id)) => {
                    return GameState::Game(id);
                }
                Some(ControllerAction::OpenEditor(p)) => {
                    return GameState::Editor(p);
                }
                _ => {}
            }
        }
        if let Some(_r) = e.render_args() {
            g2d.draw(
                encoder,
                lowres_color,
                lowres_stencil,
                logical_viewport,
                |c, g| game.render(c, cache, g),
            );
            encoder.draw(&slice, lowres_pso, lowres_data);
            encoder.flush(device);
            if C::NEEDS_HI_RES {

                g2d.draw(
                    encoder,
                    output_color,
                    output_stencil,
                    physical_viewport,
                    |c, g| game.render_hires(c, cache, g),
                );
            }
            encoder.flush(device);
        }

        if let Some(_) = e.after_render_args() {
            device.cleanup();
        }
    }
    unreachable!()
}

gfx_defines! {
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
        color: [f32; 3] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        orig: gfx::TextureSampler<[f32; 4]> = "t_Video",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

fn get_video_rect() -> ([Vertex; 4], &'static [u16]) {
    const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];
    // it's just a square lulz
    (
        [
            Vertex {
                pos: [1., -1.],
                uv: [1., 0.0],
                color: [1.0; 3],
            },
            Vertex {
                pos: [-1., -1.],
                uv: [0.0, 0.0],
                color: [1.0; 3],
            },
            Vertex {
                pos: [-1., 1.],
                uv: [0.0, 1.],
                color: [1.0; 3],
            },
            Vertex {
                pos: [1., 1.],
                uv: [1., 1.],
                color: [1.0; 3],
            },
        ],
        INDICES,
    )
}

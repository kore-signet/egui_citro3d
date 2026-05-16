use citro3d::{
    Instance, attrib,
    macros::include_shader,
    render::Target,
    shader::{self, Library, Program},
};
use ctru::{
    prelude::{Apt, Gfx, Hid},
    services::gfx::{RawFrameBuffer, Screen},
};

static SHADER_BYTES: &[u8] = include_shader!("assets/vshader.pica");

pub fn init_citro3d() -> (Instance, Library, Program) {
    let instance = citro3d::Instance::new().expect("failed to initialize Citro3D");
    let shader = shader::Library::from_bytes(SHADER_BYTES).unwrap();
    let vertex_shader = shader.get(0).unwrap();

    let program = shader::Program::new(vertex_shader).unwrap();
    (instance, shader, program)
}

pub fn bottom_target<'a>(gfx: &'a Gfx, instance: &Instance) -> (Target<'a>, usize, usize) {
    let mut bottom_screen = gfx.bottom_screen.borrow_mut();
    let RawFrameBuffer { width, height, .. } = bottom_screen.raw_framebuffer();
    let bottom_target = instance
        .render_target(width, height, bottom_screen, None)
        .expect("failed to create bottom screen render target");
    (bottom_target, width, height)
}

pub fn top_target<'a>(gfx: &'a Gfx, instance: &Instance) -> (Target<'a>, usize, usize) {
    let mut top_screen = gfx.top_screen.borrow_mut();
    let RawFrameBuffer { width, height, .. } = top_screen.raw_framebuffer();
    let top_target = instance
        .render_target(width, height, top_screen, None)
        .expect("failed to create top screen render target");
    (top_target, width, height)
}

pub fn prepare_attr_info() -> attrib::Info {
    let mut attr_info = attrib::Info::new();

    let reg0 = attrib::Register::new(0).unwrap();
    let reg1 = attrib::Register::new(1).unwrap();
    let reg2 = attrib::Register::new(2).unwrap();

    attr_info
        .add_loader(reg0, attrib::Format::Float, 2)
        .unwrap();

    attr_info
        .add_loader(reg1, attrib::Format::Float, 2)
        .unwrap();

    attr_info
        .add_loader(reg2, attrib::Format::UnsignedByte, 4)
        .unwrap();
    attr_info
}

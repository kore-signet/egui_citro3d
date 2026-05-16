use citro3d::{
    citro3d_sys,
    math::FVec4,
    shader::{Library, Program},
};

use crate::texdelta;

use super::configure_texenvs;

#[cfg(feature = "dbg_printlns")]
use ctru::prelude::KeyPad;

use super::TexAndData;

use std::collections::HashMap;

use citro3d::Instance;

#[cfg(feature = "dbg_printlns")]
use ctru::prelude::Hid;

pub(crate) const TWOVECS_BOTTOM: [[f32; 4]; 2] =
    [[1.0, 0.0, -2.0 / 240.0, 0.0], [1.0, 0.0, 0.0, -2.0 / 320.0]];
pub(crate) const TWOVECS_TOP: [[f32; 4]; 2] =
    [[1.0, 0.0, -2.0 / 240.0, 0.0], [1.0, 0.0, 0.0, -2.0 / 400.0]];

pub struct Renderer {
    instance: Instance,
    texmap: egui::ahash::HashMap<egui::TextureId, TexAndData>,
    projection_uniform_idx: citro3d::uniform::Index,
    attr_info: citro3d::attrib::Info,
    shader: Library,
    program: Program, // twovecs: [[f32; 4]; 2]
}

impl Renderer {
    pub fn new(
        projection_uniform_idx: citro3d::uniform::Index,
        attr_info: citro3d::attrib::Info,
        program: Program,
        shader: Library,
        instance: Instance,
    ) -> Self {
        Renderer {
            texmap: egui::ahash::HashMap::default(),
            projection_uniform_idx,
            attr_info,
            program,
            instance,
            shader,
        }
    }

    pub fn render(
        &mut self,
        ctx: &egui::Context,
        render_target: &mut citro3d::render::Target<'_>,
        twovecs: [[f32; 4]; 2],
        out: egui::FullOutput,
    ) {
        #[cfg(feature = "dbg_printlns")]
        if !out.textures_delta.set.is_empty() {
            println!("Adding/Patching {} Textures", out.textures_delta.set.len());
        }
        #[cfg(feature = "dbg_printlns")]
        if hid.keys_down().contains(KeyPad::B) {
            println!("Rendering {} shapes", out.shapes.len());
        }
        #[cfg(feature = "dbg_printlns")]
        if hid.keys_down().contains(KeyPad::Y) {
            println!("{:#?}", out.shapes);
        }

        texdelta::texdelta(&mut self.texmap, out.textures_delta.set);
        let tessel = ctx.tessellate(out.shapes, 1.0);
        self.instance.bind_program(&self.program);

        self.instance.render_frame_with(|mut instance| {
            unsafe {
                citro3d_sys::C3D_AlphaBlend(
                    ctru_sys::GPU_BLEND_ADD,
                    ctru_sys::GPU_BLEND_ADD,
                    ctru_sys::GPU_ONE,
                    ctru_sys::GPU_ONE_MINUS_SRC_ALPHA,
                    ctru_sys::GPU_ONE,
                    ctru_sys::GPU_ONE_MINUS_SRC_ALPHA,
                )
            };

            render_target.clear(citro3d::render::ClearFlags::ALL, 0xFF_00_00_00, 0);
            // let mut last_christmas_i_gave_you_my = None;

            instance
                .select_render_target(&*render_target)
                .expect("wharg");

            instance.bind_vertex_uniform(self.projection_uniform_idx, twovecs_to_uniform(twovecs));
            instance.set_attr_info(&self.attr_info);
            #[cfg(feature = "dbg_printlns")]
            if hid.keys_down().contains(KeyPad::B) {
                println!("Rendering {} prims", tessel.len());
            }
            for t in tessel.into_iter() {
                let mesh = match t.primitive {
                    egui::epaint::Primitive::Mesh(mesh) => mesh,
                    egui::epaint::Primitive::Callback(_) => {
                        continue;
                    }
                };
                let TexAndData { tex, data } = self.texmap.get_mut(&mesh.texture_id).unwrap();
                tex.bind(0);
                configure_texenvs::configure_texenv(&mut instance, data);
                for mesh in mesh.split_to_u16() {
                    #[cfg(feature = "dbg_printlns")]
                    if hid.keys_down().contains(KeyPad::X) {
                        println!("Tex  : {}x{}@{}", tex.width, tex.height, tex.format);
                        println!("Verts: ");
                        for vert in &mesh.vertices {
                            println!("{:?}", vert);
                        }
                        println!("Indices: ");
                        for arr in mesh.indices.chunks_exact(3) {
                            println!("({} {} {})", arr[0], arr[1], arr[2]);
                        }
                    }
                    use crate::cimm::attr;
                    use crate::cimm::imm;
                    imm(|| {
                        for i in mesh.indices {
                            let egui::epaint::Vertex { pos, uv, color } = mesh.vertices[i as usize];

                            attr([pos.x, pos.y, 0.0, 0.0]);
                            attr([uv.x, uv.y, 0.0, 0.0]);
                            attr([
                                color.r() as f32 / 255.0,
                                color.g() as f32 / 255.0,
                                color.b() as f32 / 255.0,
                                color.a() as f32 / 255.0,
                                // 0.0
                            ]);
                        }
                    });
                }

                // instance.set
                unsafe {
                    use citro3d_sys::{C3D_DirtyTexEnv, C3D_GetTexEnv};
                    let te = C3D_GetTexEnv(0);
                    C3D_DirtyTexEnv(te);
                }
            }
        });
        for remove in out.textures_delta.free {
            self.texmap.remove(&remove);
        }
    }
}

pub(crate) fn twovecs_to_uniform(twovecs_bottom: [[f32; 4]; 2]) -> citro3d::uniform::Uniform {
    citro3d::uniform::Uniform::Float2([
        FVec4::from_raw(citro3d_sys::C3D_FVec {
            c: twovecs_bottom[0],
        }),
        FVec4::from_raw(citro3d_sys::C3D_FVec {
            c: twovecs_bottom[1],
        }),
    ])
}

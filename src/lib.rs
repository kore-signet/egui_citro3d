pub mod cimm;
pub mod texture;

pub(crate) mod common;
pub(crate) mod configure_texenvs;
pub(crate) mod create_viewports;
pub(crate) mod ime;
pub(crate) mod input;
pub(crate) mod render;
pub(crate) mod texdelta;

use std::{collections::HashMap, ops::Deref};

use citro3d::{
    Instance, citro3d_sys,
    shader::{Library, Program},
};
use ctru::{
    prelude::{Hid, KeyPad},
    services::{apt::Apt, gfx::Gfx},
};
use derive_more::derive::From;
use egui::{Rect, ViewportId, ViewportIdMap, ViewportInfo};

use crate::{
    common::{init_citro3d},
    ime::ImeState,
    render::{Renderer, TWOVECS_BOTTOM, TWOVECS_TOP},
    texture::Texture,
};

pub struct TexAndData {
    tex: Texture,
    data: ImgDat,
}

#[derive(From)]
enum ImgDat {
    Rgba8(Vec<u32>),
    Alpha8(Vec<u8>),
}

impl Deref for ImgDat {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        match self {
            ImgDat::Rgba8(vec) => bytemuck::cast_slice(&vec[..]),
            ImgDat::Alpha8(vec) => bytemuck::cast_slice(&vec[..]),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Top,
    Bottom,
}

#[derive(Clone, Copy)]
pub struct Specifics<'a> {
    pub hid: &'a Hid,
    pub screen: Screen,
}

pub struct Targets<'a> {
    viewports: ViewportIdMap<ViewportInfo>,
    top: RenderTarget<'a>,
    bottom: RenderTarget<'a>,
}

impl<'a> Targets<'a> {
    pub fn new(gfx: &'a Gfx, instance: &mut Instance) -> Self {
        let (bottom_target, bottom_height, bottom_width) = common::bottom_target(&gfx, &instance);
        let (top_target, top_height, top_width) = common::top_target(&gfx, &instance);

        let bottom_screen_size = egui::vec2(bottom_width as f32, bottom_height as f32);
        let bottom_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, bottom_screen_size);
        let top_screen_size = egui::vec2(top_width as f32, top_height as f32);
        let top_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, top_screen_size);
        let bottom_viewport_id = egui::ViewportId::from_hash_of("bottom_viewport");
        let top_viewport_id = egui::ViewportId::from_hash_of("top_viewport");
        let viewports = create_viewports::create_viewports(
            bottom_screen_size,
            bottom_rect,
            top_screen_size,
            top_rect,
            bottom_viewport_id,
            top_viewport_id,
        );

        Targets {
            viewports,
            top: RenderTarget {
                target: top_target,
                rect: top_rect,
                viewport_id: top_viewport_id,
                focus: false,
            },
            bottom: RenderTarget {
                target: bottom_target,
                rect: bottom_rect,
                viewport_id: bottom_viewport_id,
                focus: true,
            },
        }
    }
}

pub struct RenderTarget<'a> {
    target: citro3d::render::Target<'a>,
    rect: Rect,
    viewport_id: ViewportId,
    focus: bool,
}

impl<'a> RenderTarget<'a> {
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        events: Vec<egui::Event>,
        viewports: ViewportIdMap<ViewportInfo>,
        c: &mut impl FnMut(&mut egui::Ui),
    ) -> egui::output::FullOutput {
        ctx.run_ui(
            egui::RawInput {
                events,
                viewport_id: self.viewport_id,
                viewports,
                focused: self.focus,
                max_texture_side: Some(1024),
                screen_rect: Some(self.rect),
                ..Default::default()
            },
            c,
        )
    }
}

pub struct EguiRenderer<'a> {
    key_mapping: Vec<(KeyPad, (egui::Key, egui::Modifiers))>,
    gfx: &'a Gfx,
    hid: &'a mut Hid,
    apt: &'a Apt,
    // instance: Instance,
    // shader: Library,
    // program: Program,
    ctx: egui::Context,
    targets: Targets<'a>,
    ime: ImeState,
    last_pos: egui::Pos2,
    renderer: Renderer,
}

impl<'a> EguiRenderer<'a> {
    pub fn new(hid: &'a mut Hid, gfx: &'a Gfx, apt: &'a Apt, key_mapping: &[(KeyPad, (egui::Key, egui::Modifiers))]) -> Self {
        let (mut instance, shader, program) = init_citro3d();

        let ctx = egui::Context::default();
        ctx.options_mut(|opts| {
            opts.reduce_texture_memory = true;
            opts.theme_preference = egui::ThemePreference::Dark;
        });
        ctx.set_embed_viewports(false);

        // instance.bind_program(&program);
        let projection_uniform_idx = program
            .get_uniform("transform")
            .expect("No transform uniform?");
        let attr_info = common::prepare_attr_info();

        let targets = Targets::new(&gfx, &mut instance);

        let renderer =
            Renderer::new(projection_uniform_idx, attr_info, program, shader, instance);

        unsafe {
            //If you delete this call, faces *will* be culled
            citro3d_sys::C3D_CullFace(ctru_sys::GPU_CULL_NONE);
        }

        EguiRenderer {
            ime: ImeState::new(),
            key_mapping: key_mapping.to_vec(),
            gfx,
            targets,
            hid,
            apt,
            ctx,
            last_pos: egui::Pos2::default(),
            renderer
        }
    }

    pub fn render_frame(
        &mut self,
        mut top_ui: impl FnMut(&mut egui::Ui),
        mut bottom_ui: impl FnMut(&mut egui::Ui),
    ) -> bool {
        self.hid.scan_input();
        let (mut events, start_button) =
            input::handle_input(self.hid, &self.key_mapping, &mut self.last_pos);
        if start_button {
            return false;
        }

        self.ime.part_a(self.gfx, self.apt, &mut events);

        let bottom_out = self.targets.bottom.render(
            &self.ctx,
            events,
            self.targets.viewports.clone(),
            &mut bottom_ui,
        );

        self.ime.part_b(&bottom_out);

        self.renderer.render(
            &self.ctx,
            &mut self.targets.bottom.target,
            TWOVECS_BOTTOM,
            bottom_out,
        );

        let top_out = self.targets.top.render(
            &self.ctx,
            Vec::new(),
            self.targets.viewports.clone(),
            &mut top_ui,
        );

        self.renderer.render(
            &self.ctx,
            &mut self.targets.top.target,
            TWOVECS_TOP,
            top_out,
        );

        true
    }
}

//!
//! The entities in the playground
//! HanishKVC, 2022
//!

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use crate::sdlx::SdlX;
use crate::playdata::PositionsUpdate;


const ENTITY_WIDTH: u32 = 16;
const ENTITY_HEIGHT: u32 = 16;

pub const SCREEN_WIDTH: u32 = 1024;
pub const SCREEN_HEIGHT: u32 = 600;
pub const SCREEN_COLOR_BG: Color = Color::RGB(20, 200, 20);

pub const FRAMES_PER_SEC: usize = 24;


pub fn screen_color_bg_rel(r: u8, g: u8, b: u8) -> Color {
    Color {
        r: SCREEN_COLOR_BG.r+r,
        g: SCREEN_COLOR_BG.g+g,
        b: SCREEN_COLOR_BG.b+b,
        a: SCREEN_COLOR_BG.a,
    }
}

type _PosInt = i32;

pub mod gentity;
pub mod team;


#[derive(Debug)]
pub(crate) struct Entities<'a> {
    ateam: team::Team<'a>,
    bteam: team::Team<'a>,
}

impl<'a> Entities<'a> {

    pub fn new(anplayers: i32, bnplayers: i32, font: &'a Font) -> Entities<'a> {
        Entities {
            ateam: team::Team::new("ateam", Color::RED, anplayers, font),
            bteam: team::Team::new("bteam", Color::BLUE, bnplayers, font),
        }
    }

    pub fn update(&mut self, pu: PositionsUpdate, babsolute: bool) {
        self.ateam.update(pu.ateampositions, babsolute);
        self.bteam.update(pu.bteampositions, babsolute);
    }

    pub fn next_frame(&mut self) {
        self.ateam.next_frame();
        self.bteam.next_frame();
    }

    fn draw_pitch(&self, sx: &mut SdlX) {
        sx.nn_line(0.02, 0.02, 0.98, 0.02, Color::WHITE);
        sx.nn_line(0.02, 0.02, 0.02, 0.98, Color::WHITE);
        sx.nn_line(0.02, 0.98, 0.98, 0.98, Color::WHITE);
        sx.nn_line(0.98, 0.02, 0.98, 0.98, Color::WHITE);
        sx.nn_line(0.50, 0.02, 0.50, 0.98, Color::GRAY);
    }

    pub fn draw(&self, sx: &mut SdlX) {
        self.draw_pitch(sx);
        self.ateam.draw(sx);
        self.bteam.draw(sx);
    }

}

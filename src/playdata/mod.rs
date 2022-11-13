//!
//! Data related to play
//! HanishKVC, 2022
//!


#[derive(Debug)]
pub struct PositionsUpdate {
    pub stimemsg: String,
    pub ball: (f32, f32),
    pub ateampositions: Vec<(i32, f32, f32)>,
    pub bteampositions: Vec<(i32, f32, f32)>,
}

impl PositionsUpdate {

    pub fn new() -> PositionsUpdate {
        PositionsUpdate {
            stimemsg: String::new(),
            ball: (0.0,0.0),
            ateampositions: Vec::new(),
            bteampositions: Vec::new(),
        }
    }

}

pub trait PlayData {

    fn setup(&mut self, fps: f32);

    fn next_frame_is_record_ready(&mut self) -> bool;

    fn next_record(&mut self) -> PositionsUpdate;

    fn bdone(&self) -> bool;

}

pub mod random;
pub mod rcg;

//!
//! 2D Playback and look at captured game data
//! HanishKVC, 2022
//!

use std::env;
use std::time;

use sdl2::pixels::Color;
use sdl2::ttf::Font;

use loggerk::{log_init, ldebug, log_d};

mod entities;
mod sdlx;
mod playdata;
use playdata::rcg::Rcg;
use playdata::random::RandomData;
use playdata::{PlayData, rclive};
use playdata::rclive::RCLive;
use sdlx::SdlX;
use entities::PGEntities;

mod testlib;
mod keys;
mod proc;

struct Gui<'a> {
    /// Whether help msgbox should be shown or not in the current frame
    showhelp: bool,
    /// Pause the playback
    pause: bool,
    /// Current frame number
    frame: usize,
    /// Time alloted per frame
    frametime: time::Duration,
    /// fps tracking: frame number wrt prev second
    fpsframe: usize,
    /// fps tracking: time wrt prev second
    fpstime: time::Instant,
    /// fps tracking: actually achieved fps
    actualfps: usize,
    /// the time at begining of processing wrt current frame
    curframetime: time::Instant,
    /// Playground entities
    pgentities: PGEntities<'a>,
    /// Playdata source
    pdata: Box<dyn PlayData>,
    /// Show Passes summary
    showpasses: bool,
}

impl<'a> Gui<'a> {

    fn calc_frametime(fps: f32) -> time::Duration {
        time::Duration::from_millis((1000.0/fps).round() as u64)
    }

    /// Sync up fps to the seconds per record of the playdata source
    #[cfg(feature="inbetween_frames")]
    fn sync_up_fps_to_spr(&mut self) {
        self.fps_adjust(1.0);
    }

    #[cfg(not(feature="inbetween_frames"))]
    fn sync_up_fps_to_spr(&mut self) {
        let spr = self.pdata.seconds_per_record();
        let fpsadj = (1.0/spr)/self.pgentities.fps();
        self.fps_adjust(fpsadj);
    }

}

impl<'a> Gui<'a> {

    fn new(fps: f32, font: &'a Font) -> Gui<'a> {
        // PGEntities
        let mut pgentities = entities::PGEntities::new(entities::PITCH_RECT, 11, 11, fps, font);
        pgentities.adjust_teams();
        // Playdata source
        let clargs = env::args().collect::<Vec<String>>();
        let (pdata, showhelp) = pdata_source(&clargs, pgentities.fps());

        let ctime = time::Instant::now();
        let mut gui = Gui {
            showhelp: showhelp,
            pause: false,
            frame: 0,
            frametime: Self::calc_frametime(fps),
            fpsframe: 0,
            fpstime: ctime,
            actualfps: 0,
            curframetime: ctime,
            pgentities: pgentities,
            pdata: pdata,
            showpasses: false,
        };
        // sync up fps to spr
        gui.sync_up_fps_to_spr();
        return gui;
    }

    /// Update gui internal state, as needed, when fps requested by user/playdata source/... changes
    fn internal_fps_changed(&mut self, fps: f32) {
        self.frametime = Self::calc_frametime(fps);
    }

    /// Adjust the fps to be used wrt the program.
    /// It inturn takes care of keeping gui internal logic, pgentities and pdata in sync wrt fps changes
    fn fps_adjust(&mut self, ratio: f32) {
        self.pgentities.fps_adjust(ratio);
        self.pdata.fps_changed(self.pgentities.fps());
        self.internal_fps_changed(self.pgentities.fps());
        eprintln!("INFO:PPGND:Main:Fps:{}", self.pgentities.fps());
    }

    /// Update internal state, wrt/related-to begining of a new frame
    fn next_frame(&mut self) {
        self.frame += 1;
        self.curframetime = time::Instant::now();
        let dtime = self.curframetime.duration_since(self.fpstime);
        if dtime > time::Duration::from_millis(1000) {
            self.fpstime = self.curframetime;
            self.actualfps = self.frame - self.fpsframe;
            self.fpsframe = self.frame;
        }
    }

    /// Consume any frame time remaining wrt current frame, by sleeping
    fn consume_frametime(&mut self) {
        let ctime = time::Instant::now();
        let consumedtime = ctime.duration_since(self.curframetime);
        if self.frametime > consumedtime {
            let dtime = self.frametime - consumedtime;
            std::thread::sleep(dtime);
        }
    }

}

fn show_help(sx: &mut SdlX) {
    let shelp = "** Help **\n\
    \n\
    larrow: seek back\n\
    rarrow: seek forward\n\
    f/F:    change fps\n\
    p:      pause playback\n\
    ss:     show/hide stamina\n\
    sa:     show/hide actions\n\
    sb:     show/hide ball\n\
    c1:     RCLive kick-off\n\
    c0:     RCLive init hs\n\
    h:      hide/unhide help\n\
    \n\
    playbackpgnd <live [addr]> | <path/file.rcg>\n\
    ...                   Save Nature Save Earth";

    let vhelp: Vec<&str> = shelp.split('\n').collect();
    sx.n_msgbox((0.3,0.2, 0.4,0.6), vhelp, Color::BLUE);

}

#[allow(dead_code)]
fn test_me(font: &Font) {
    testlib::test_ncolor();
    testlib::test_gentity(font);
}

fn identify() {
    println!("Playback Playground");
    if cfg!(feature = "inbetween_frames") {
        println!("INFO:PPGND:Mode: InBetween Frames");
    } else {
        println!("INFO:PPGND:Mode: OnlyProvided Frames");
    }
}

///
/// Setup the playdata source based on passed args.
/// * if no args, then start the random playdata source
/// * if live passed as 1st arg to program, then try to
///   connect to a running rcssserver.
///   * if a 2nd argument is passed, use it has the nw
///     address of the server to connect to.
///   * else use the default address specified in rclive.
/// * else use the 1st argument as the rcg file to playback.
///
/// Return the playdata source and whether help msgbox should be shown
///
fn pdata_source(vargs: &Vec<String>, fps: f32) -> (Box<dyn PlayData>, bool) {
    let src;
    if vargs.len() > 1 {
        src = vargs[1].as_str();
    } else {
        src = "";
    }
    if src == "live" {
        let nwaddr;
        if vargs.len() > 2 {
            nwaddr = vargs[2].as_str();
        } else {
            nwaddr = rclive::NWADDR_DEFAULT;
        }
        let pdrcl = RCLive::new(nwaddr);
        return (Box::new(pdrcl), false);
    } else if src.len() > 0 {
        let pdrcg = Rcg::new(src, fps);
        return (Box::new(pdrcg), false);
    } else {
        let pdrandom = RandomData::new(1.0/24.0, 11, 11);
        return (Box::new(pdrandom), true);
    }
}


fn main() {
    log_init();
    identify();

    // SDL related setup
    let ttfx = sdl2::ttf::init().unwrap();
    let font = ttfx.load_font(sdlx::TTF_FONT, sdlx::TTF_FONT_SIZE);
    if font.is_err() {
        let err = font.err().unwrap();
        eprintln!("ERRR:PPGND:Loading font[{}], install it or update font in sdlx.rs:{}", sdlx::TTF_FONT, err);
        std::process::exit(10);
    }
    let font = font.unwrap();
    let mut sx = sdlx::SdlX::init_plus(entities::SCREEN_WIDTH, entities::SCREEN_HEIGHT, false);

    // Get the gui program related entity
    let mut gui = Gui::new(entities::FRAMES_PER_SEC as f32, &font);

    // The main loop of the program starts now
    let mut dcolor = 20;
    let mut skey = String::new();
    'mainloop: loop {
        gui.next_frame();
        // Clear the background
        sx.wc.set_draw_color(entities::screen_color_bg_rel(dcolor, 0, 0));
        sx.wc.clear();
        sx.n_msg(entities::MSG_FPS_POS.0, entities::MSG_FPS_POS.1, &format!("[{}] [{},{}]", skey, &gui.pgentities.fps().round(), gui.actualfps), sdlx::Color::BLUE);

        // handle any pending/queued program events
        'eventloop: loop {
            let prgev= keys::get_programevents(&mut sx, &mut skey);
            match prgev {
                keys::ProgramEvent::None => break 'eventloop,
                keys::ProgramEvent::Pause => gui.pause = !gui.pause,
                keys::ProgramEvent::BackgroundColorChange => dcolor = dcolor.wrapping_add(20),
                keys::ProgramEvent::ToggleShowHelp => gui.showhelp = !gui.showhelp,
                keys::ProgramEvent::ToggleShowBall => gui.pgentities.showball = !gui.pgentities.showball,
                keys::ProgramEvent::ToggleShowActions => gui.pgentities.toggle_bshowactions(),
                keys::ProgramEvent::ToggleShowStamina => gui.pgentities.toggle_bstamina(),
                keys::ProgramEvent::SeekBackward => gui.pdata.seek(-50),
                keys::ProgramEvent::SeekForward => gui.pdata.seek(50),
                keys::ProgramEvent::AdjustFPS(ratio) => {
                    gui.fps_adjust(ratio);
                },
                keys::ProgramEvent::SendRecordCoded(code) => gui.pdata.send_record_coded(code),
                keys::ProgramEvent::DumpPGEntities => eprintln!("DBUG:PPGND:Main:Entities:{:#?}", gui.pgentities),
                keys::ProgramEvent::DumpPasses => {
                    gui.pgentities.passes.summary();
                    gui.showpasses = !gui.showpasses;
                }
                keys::ProgramEvent::Quit => break 'mainloop,
                keys::ProgramEvent::NeedMore => (),
            }
        }

        // Update the entities
        if !gui.pause {
            if !gui.pdata.bdone() {
                if cfg!(feature = "inbetween_frames") {
                    if gui.pdata.next_frame_is_record_ready() {
                        let pu = gui.pdata.next_record();
                        ldebug!(&format!("DBUG:{:?}", pu));
                        gui.pgentities.update(pu, false, gui.pdata.seconds_per_record() * gui.pgentities.fps());
                        //eprintln!("DBUG:PPGND:Main:{}:Update called", _frame);
                    }
                    // TODO: Need to let this run for Fps frames ideally, even after bdone is set
                    // Or Rcg needs to be udpated to set bdone after a second of ending or so ...
                    gui.pgentities.next_frame();
                    //eprintln!("DBUG:PPGND:Main:{}:NextFrame called", _frame);
                } else {
                    let pu = gui.pdata.next_record();
                    gui.pgentities.update(pu, true, 0.0);
                }
            }
        }

        // Draw entities
        gui.pgentities.draw(&mut sx);
        if gui.showhelp {
            show_help(&mut sx);
        }

        // Draw info
        if gui.showpasses {
            gui.pgentities.passes.summary_sdl(&mut sx);
        }

        // Present screen update to user
        sx.wc.present();

        // consume any remaining frame time
        gui.consume_frametime();
    }

}

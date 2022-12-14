//!
//! Simulated objects
//! HanishKVC, 2022
//!

use loggerk::{ldebug, log_d};

#[derive(Debug)]
/// A interpolated ball
pub struct VirtBall {
    /// The raw data records
    vdata: Vec<String>,
    /// current index into the data records
    vin: usize,
    /// Timestamp wrt next action/ball movement change
    ltime: usize,
    /// Starting position of next action/ball movement change
    lpos: (f32, f32),
    /// Position of the ball as it stands now
    cpos: (f32, f32),
    /// Base amount of change to apply wrt ball position, per step
    mov: (f32, f32),
    /// The time stamp for which position was generated in the last call
    lastgentime: usize,
    /// unit step size
    stepsize: f32,
    /// % of steps done wrt current interpolation block in 0.0-1.0 form
    stepdone: f32,
}

impl VirtBall {

    pub fn new(fname: &str) -> VirtBall {
        let sdata = String::from_utf8(std::fs::read(fname).unwrap()).unwrap();
        let tdata = sdata.split('\n').collect::<Vec<&str>>();
        let mut vdata = Vec::new();
        for data in tdata {
            vdata.push(data.to_string());
        }
        VirtBall {
            vdata: vdata,
            vin: 0,
            ltime: 0,
            lpos: (0.0, 0.0),
            cpos: (0.0, 0.0),
            mov: (0.0, 0.0),
            lastgentime: 0,
            stepsize: 0.0,
            stepdone: 0.0,
        }
    }

    fn next_cpos(&mut self) {
        self.cpos = (self.cpos.0 + self.mov.0, self.cpos.1 + self.mov.1);
    }

    fn extract_nextdata(&mut self) -> bool {
        let sdata = &self.vdata[self.vin];
        self.vin += 1;
        if sdata.trim().len() == 0 {
            return false;
        }
        let sdata = sdata.split(',').collect::<Vec<&str>>();
        self.ltime = sdata[0].parse().unwrap();
        let fx = sdata[1].parse().unwrap();
        let fy = sdata[2].parse().unwrap();
        self.lpos = (fx, fy);
        return true;
    }

    ///
    /// Calculate the interpolated position wrt each requested time.
    /// If the last time is repeated again, the same position is sent.
    /// If there is no more data, keep the ball moving in the direction
    /// it already is.
    ///
    /// It uses the current position and position of the ball wrt the
    /// immidiate next action that will be there in the game future, to
    /// help interpoloate the ball positions. This calculation is repeated/
    /// done when ever the ball (or rather playback) has just gone past a
    /// known game action time, wrt the next segment.
    ///
    /// MAYBE: Add support for non linear interpolated movement later.
    ///
    pub fn next_record(&mut self, ctime: usize) -> (f32, f32) {
        if ctime == self.lastgentime {
            return self.cpos;
        }
        self.lastgentime = ctime;
        while ctime > self.ltime {
            if self.vin >= self.vdata.len() {
                break;
            }
            if !self.extract_nextdata() {
                break;
            }
            let dt = self.ltime as isize - ctime as isize;
            if dt < 0 {
                continue;
            } else if dt == 0 {
                self.cpos = self.lpos;
                return self.lpos;
            }
            self.stepsize = 1.0/dt as f32;
            self.stepdone = 0.0;
            let dx = (self.lpos.0 - self.cpos.0)/(dt as f32 +1.0);
            let dy = (self.lpos.1 - self.cpos.1)/(dt as f32 +1.0);
            self.mov = (dx,dy);
        }
        self.next_cpos();
        self.cpos
    }

    /// Try seek, but in a blind way for now
    /// As action data is not at same nor uniform granularity like time data,
    /// so the current seek logic below wont be perfect.
    ///
    /// seekdelta: time delta
    ///
    pub fn seek(&mut self, seekdelta: isize) {
        let mut newindex = self.vin as isize + seekdelta;
        if newindex < 0 {
            newindex = 0;
        }
        self.vin = newindex as usize;
        ldebug!(&format!("DBUG:PPGND:SimObjs:VirtBall:Seek:{}:{}", self.lastgentime as isize-seekdelta, self.vdata[self.vin]));
        self.extract_nextdata();
    }

}

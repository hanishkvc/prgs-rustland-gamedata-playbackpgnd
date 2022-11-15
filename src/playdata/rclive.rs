//!
//! Act as a monitor for robocup soccer sim
//! HanishKVC, 2022
//!

use std::net::UdpSocket;
use std::time;

use tokensk::TStrX;

use crate::sdlx::XSpaces;

use super::{PlayData, PlayUpdate};

const OWN_ADDRESS: &str = "0.0.0.0:6600";
const READ_TIMEOUT_MS: u64 = 500;


/// Help act as a simple monitor client for RoboCup Sim
pub struct RCLive {
    skt: UdpSocket,
    /// The robocup server address to communicate to.
    srvraddr: String,
    /// Help tokenise recieved data.
    tstrx: TStrX,
    /// Help convert from Robocups pitch space to normal space.
    r2n: XSpaces,
}

impl RCLive {

    pub fn new(addr: &str) -> RCLive {
        let skt = UdpSocket::bind(OWN_ADDRESS).unwrap();
        skt.set_read_timeout(Some(time::Duration::from_millis(READ_TIMEOUT_MS))).unwrap();
        let sinit = "(dispinit version 5)\r\n";
        skt.send_to(sinit.as_bytes(), addr).unwrap();
        eprintln!("DBUG:PPGND:RCLive:New:{:?}", skt);
        let rrect = ((-55.0, -37.0), (55.0, 37.0));
        let nrect = ((0.0,0.0), (1.0,1.0));
        let mut tstrx = TStrX::new();
        tstrx.flags.string_canbe_asubpart = true;
        tstrx.delims.bracket_begin = '{';
        tstrx.delims.bracket_end = '}';
        tstrx.delims.string = '^';
        RCLive {
            skt: skt,
            srvraddr: addr.to_string(),
            tstrx: tstrx,
            r2n: XSpaces::new(rrect, nrect),
        }
    }

}

impl PlayData for RCLive {
    fn seconds_per_record(&self) -> f32 {
        return 0.05;
    }

    fn fps_changed(&mut self, _fps: f32) {
    }

    fn next_frame_is_record_ready(&mut self) -> bool {
        return true;
    }

    fn next_record(&mut self) -> super::PlayUpdate {
        let mut pu = PlayUpdate::new();
        let mut buf = [0u8; 8196];
        let gotr = self.skt.recv_from(&mut buf);
        if gotr.is_err() {
            let err = gotr.unwrap_err();
            if err.kind() == std::io::ErrorKind::WouldBlock {
                eprintln!("WARN:PPGND:RCLive:No data...");
                return pu;
            } else {
                panic!("ERRR:PPGND:RCLive:Unexpected error:{}", err);
            }
        }
        let sbuf = String::from_utf8_lossy(&buf);
        eprintln!("DBUG:PPGND:RCLive:Got:{:?}:{}", gotr, &sbuf);
        let mut tstr = self.tstrx.from_str(&sbuf, true);
        tstr.peel_bracket('{').unwrap();
        let toks = tstr.tokens_vec(',', true, true).unwrap();
        //eprintln!("DBUG:PPGND:RCLive:Got:Toks:{:#?}", toks);
        for tok in toks {
            if tok.starts_with("\"ball\"") {
                let (_b,d) = tok.split_once(':').unwrap();
                let mut tstr = self.tstrx.from_str(d, true);
                tstr.peel_bracket('{').unwrap();
                let toksl2 = tstr.tokens_vec(',', true, true).unwrap();
                let mut fx = 0.0;
                let mut fy = 0.0;
                for tokl2 in toksl2 {
                    let (k,v) = tokl2.split_once(':').unwrap();
                    if k == "\"x\"" {
                        fx = v.parse().unwrap();
                    }
                    if k == "\"y\"" {
                        fy = v.parse().unwrap();
                    }
                }
                let (fx,fy) = self.r2n.d2o((fx,fy));
                pu.ball = (fx, fy);
                continue;
            }
            let mut tstr;
            if tok.starts_with("\"players\"") {
                let (_p,d) = tok.split_once('[').unwrap();
                tstr = self.tstrx.from_str(d, true);
            } else if !tok.starts_with("{\"side\"") {
                continue;
            } else {
                tstr = self.tstrx.from_str(&tok, true);
            }
            tstr.peel_bracket('{').unwrap();
            let toksl2 = tstr.tokens_vec(',', true, true).unwrap();
            //eprintln!("DBUG:PPGND:RCLive:Got:Toks:{:#?}", toksl2);
            let mut pnum = 0;
            let mut fx = 0.0;
            let mut fy = 0.0;
            let mut side = String::new();
            for tokl2 in toksl2 {
                let (k,v) = tokl2.split_once(':').unwrap();
                if k == "\"side\"" {
                    side = v.to_string();
                }
                if k == "\"unum\"" {
                    pnum = v.parse().unwrap();
                }
                if k == "\"x\"" {
                    fx = v.parse().unwrap();
                }
                if k == "\"y\"" {
                    fy = v.parse().unwrap();
                }
            }
            let (fx,fy) = self.r2n.d2o((fx,fy));
            if side.chars().nth(1).unwrap() == 'l' {
                pu.ateampositions.push((pnum-1, fx, fy));
            } else {
                pu.bteampositions.push((pnum-1, fx, fy));
            }
        }
        eprintln!("DBUG:PPGND:RCLive:Got:Pu:{:?}", pu);
        pu
    }

    fn seek(&mut self, _seekdelta: isize) {
        return;
    }

    fn bdone(&self) -> bool {
        return false;
    }

    fn send_record(&mut self, buf: &[u8]) {
        self.skt.send_to(buf, &self.srvraddr).unwrap();
        eprintln!("DBUG:PPGND:RCLive:Sent:{:?}:To:{:?}-{:?}", buf, self.skt, self.srvraddr);
    }

}

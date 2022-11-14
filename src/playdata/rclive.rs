//!
//! Act as a monitor for robocup soccer sim
//! HanishKVC, 2022
//!

use std::net::UdpSocket;

use tokensk::{TStr, TStrX};

use crate::sdlx::XSpaces;

use super::{PlayData, PlayUpdate};

pub struct RCLive {
    skt: UdpSocket,
    srvraddr: String,
    stx: TStrX,
    r2n: XSpaces,
}

impl RCLive {

    pub fn new(addr: &str) -> RCLive {
        let skt = UdpSocket::bind("0.0.0.0:6600").unwrap();
        let sinit = "(dispinit version 5)\r\n";
        skt.send_to(sinit.as_bytes(), addr).unwrap();
        eprintln!("DBUG:PPGND:RCLive:New:{:?}", skt);
        let rrect = ((-55.0, -37.0), (55.0, 37.0));
        let nrect = ((0.0,0.0), (1.0,1.0));
        RCLive {
            skt: skt,
            srvraddr: addr.to_string(),
            stx: TStrX::new(),
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
        let sbuf = String::from_utf8_lossy(&buf);
        eprintln!("DBUG:PPGND:RCLive:Got:{:?}:{}", gotr, &sbuf);
        let mut tstr = TStr::from_str(&sbuf, true);
        tstr.delims.bracket_begin = '{';
        tstr.delims.bracket_end = '}';
        tstr.delims.string = '^';
        tstr.peel_bracket('{').unwrap();
        let toks = tstr.tokens_vec(',', true, true).unwrap();
        eprintln!("DBUG:PPGND:RCLive:Got:Toks:{:#?}", toks);
        for tok in toks {
            if tok.starts_with("\"ball\"") {
                let (_b,d) = tok.split_once(':').unwrap();
                let mut tstr = TStr::from_str(d, true);
                tstr.delims.bracket_begin = '{';
                tstr.delims.bracket_end = '}';
                tstr.delims.string = '^';
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
                tstr = TStr::from_str(d, true);
            } else if !tok.starts_with("{\"side\"") {
                continue;
            } else {
                tstr = TStr::from_str(&tok, true);
            }
            tstr.delims.bracket_begin = '{';
            tstr.delims.bracket_end = '}';
            tstr.delims.string = '^';
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
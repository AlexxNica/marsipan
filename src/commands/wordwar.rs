use chrono::DateTime;
use chrono::Duration;
use chrono::Local;
use chrono::Timelike;
use commands::prelude::*;
use nom::digit;
use state::Storage;
use std::collections::{HashMap,HashSet};
use std::time::Duration as StdDuration;
use std::sync::Mutex;

fn until(other: DateTime<Local>) -> Option<StdDuration> {
    let d = other.signed_duration_since(Local::now());
    if d < Duration::zero() {
        return None
    }
    let nanos_only = d - Duration::seconds(d.num_seconds());
    Some(StdDuration::new(d.num_seconds() as u64, nanos_only.num_nanoseconds().unwrap() as u32))
}

struct War {
    start: Instant,
    end: Instant,
    participants: HashSet<String>,
    starter: String,
}

static WARS: Storage<Mutex<HashMap<W, War>>> = Storage::new();

named!(dec<u32>, map_res!(map_res!(digit, ::std::str::from_utf8), ::std::str::FromStr::from_str));

named!(parse_ww<(u32,u32)>, do_parse!(
    tag!("at :") >>
    min: dec >>
    tag!(" for ") >>
    dur: dec >>
    (min, dur)
));

impl War {
    fn active(&self) -> bool {
        self.end <= Instant::now()
    }

    fn parse(bytes: &[u8]) -> Result<(DateTime<Local>, DateTime<Local>), String> {
        let (at, dur) = parse_ww(bytes).to_result().map_err(|_|"I don't understand.".to_string())?;
        if dur > 59 {
            return Err("Too many minutes.".to_string())
        }
        let current_time = Local::now();
        let start_time = if current_time.minute() >= at {
            current_time + Duration::hours(1)
        } else {
            current_time
        }.with_minute(at).and_then(|m|m.with_second(0))
         .and_then(|m|m.with_nanosecond(0)).ok_or("math error")?;
        Ok((start_time, start_time + Duration::minutes(dur as i64)))
    }
}

pub fn wordwar(e: Event) -> Hooks {
    let res = War::parse(e.content().as_bytes());
    e.respond(format!("{:?}", res));

    vec![]
}

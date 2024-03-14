use ctor::ctor;
use lazy_static::lazy_static;
use libc::{getrusage, rusage, RUSAGE_SELF};
use std::{env, fmt, io, mem::MaybeUninit, result, time::Instant};

type Result<T> = result::Result<T, io::Error>;

lazy_static! {
    static ref NOW: Instant = Instant::now();
}

#[ctor]
fn init_now() {
    lazy_static::initialize(&NOW);
}

pub fn realtime() -> u64 {
    NOW.elapsed().as_secs()
}

struct AppResources {
    start_time: Instant,
    command_line: String,
}

impl AppResources {
    fn new() -> Result<Self> {
        let start_time = Instant::now();
        let command_line = env::args().collect::<Vec<String>>().join(" ");

        Ok(Self {
            start_time,
            command_line,
        })
    }

    fn cputime(&self) -> Result<i64> {
        let r = unsafe {
            let mut r = MaybeUninit::<rusage>::uninit();
            if getrusage(RUSAGE_SELF, r.as_mut_ptr()) == -1 {
                return Err(io::Error::last_os_error());
            }
            r.assume_init()
        };
        Ok(r.ru_utime.tv_sec + r.ru_stime.tv_sec)
    }

    fn peakrss(&self) -> Result<i64> {
        let r = unsafe {
            let mut r = MaybeUninit::uninit();
            if getrusage(RUSAGE_SELF, r.as_mut_ptr()) == -1 {
                return Err(io::Error::last_os_error());
            }
            r.assume_init()
        };
        Ok(r.ru_maxrss)
    }
}

impl fmt::Display for AppResources {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CMD: {}\nReal time: {} sec; CPU: {} sec; Peak RSS: {:.3} GB",
            self.command_line,
            realtime(),
            self.cputime().unwrap_or_default(),
            self.peakrss().unwrap_or_default() as f64 / 1024.0 / 1024.0,
        )
    }
}

pub fn gather_app_resources() -> Result<String> {
    let app_resources = AppResources::new()?;
    Ok(format!("{}", app_resources))
}

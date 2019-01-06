#![feature(duration_extras)]

extern crate serial;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;
extern crate xmodem;

use std::time::Instant;
use std::path::PathBuf;
use std::time::Duration;

use structopt::StructOpt;
use serial::core::{BaudRate, CharSize, FlowControl, SerialDevice, SerialPortSettings, StopBits};
use xmodem::{Progress, Xmodem};

mod parsers;

use parsers::{parse_baud_rate, parse_flow_control, parse_stop_bits, parse_width};

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(short = "i", help = "Input file (defaults to stdin if not set)",
                parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short = "b", long = "baud", parse(try_from_str = "parse_baud_rate"),
                help = "Set baud rate", default_value = "115200")]
    baud_rate: BaudRate,

    #[structopt(short = "t", long = "timeout", parse(try_from_str),
                help = "Set timeout in seconds", default_value = "10")]
    timeout: u64,

    #[structopt(short = "w", long = "width", parse(try_from_str = "parse_width"),
                help = "Set data character width in bits", default_value = "8")]
    char_width: CharSize,

    #[structopt(help = "Path to TTY device", parse(from_os_str))]
    tty_path: PathBuf,

    #[structopt(short = "f", long = "flow-control", parse(try_from_str = "parse_flow_control"),
                help = "Enable flow control ('hardware' or 'software')", default_value = "none")]
    flow_control: FlowControl,

    #[structopt(short = "s", long = "stop-bits", parse(try_from_str = "parse_stop_bits"),
                help = "Set number of stop bits", default_value = "1")]
    stop_bits: StopBits,

    #[structopt(short = "r", long = "raw", help = "Disable XMODEM")]
    raw: bool,
}

fn progress_fn(_progress: Progress) {
    static mut LAST_TIME: Option<Instant> = None;
    static mut BYTES_SENT: u64 = 0;
    unsafe {
        BYTES_SENT += 128;
        LAST_TIME = match LAST_TIME {
            Some(last_time) => {
                let now = Instant::now();
                let duration = now - last_time;
                let nanos = duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as u64;
                println!(
                    "Progress: {} bytes sent at {:.2} KiB/s",
                    BYTES_SENT,
                    128.0 * 1_000_000_000.0 / 1024.0 / nanos as f64
                );
                Some(now)
            }
            None => Some(Instant::now()),
        };
    }
}

fn main() {
    use std::fs::File;
    use std::io::{self, BufReader};

    let opt = Opt::from_args();
    let mut serial = serial::open(&opt.tty_path).expect("path points to invalid TTY");

    // FIXME: Implement the `ttywrite` utility.
    let mut settings = serial.read_settings().expect("device should be valid");
    settings
        .set_baud_rate(opt.baud_rate)
        .expect("baud rate should be valid");
    settings.set_stop_bits(opt.stop_bits);
    settings.set_flow_control(opt.flow_control);
    settings.set_char_size(opt.char_width);
    serial
        .write_settings(&settings)
        .expect("settings should be valid");
    serial
        .set_timeout(Duration::from_secs(opt.timeout))
        .expect("timeout should be valid");

    if opt.raw {
        match opt.input {
            Some(ref path) => {
                let mut input = BufReader::new(File::open(path).expect("file should exist"));
                io::copy(&mut input, &mut serial).expect("io transfer should succeed");
            }
            None => {
                let mut input = io::stdin();
                io::copy(&mut input, &mut serial).expect("io transfer should succeed");
            }
        };
    } else {
        // XMODEM
        match opt.input {
            Some(ref path) => {
                let mut input = BufReader::new(File::open(path).expect("file should exist"));
                match Xmodem::transmit_with_progress(input, &mut serial, progress_fn) {
                    Ok(_) => return,
                    Err(err) => panic!("Error: {:?}", err),
                }
            }
            None => {
                let mut input = io::stdin();
                match Xmodem::transmit_with_progress(input, &mut serial, progress_fn) {
                    Ok(_) => return,
                    Err(err) => panic!("Error: {:?}", err),
                }
            }
        }
    }
}

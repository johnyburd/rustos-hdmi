mod parsers;

use serial;
use structopt;
use structopt_derive::StructOpt;
use xmodem::Xmodem;
use xmodem::Progress;

use std::path::PathBuf;
use std::time::Duration;

use structopt::StructOpt;
use serial::core::{CharSize, BaudRate, StopBits, FlowControl, SerialDevice, SerialPortSettings};

use parsers::{parse_width, parse_stop_bits, parse_flow_control, parse_baud_rate};

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(short = "i", help = "Input file (defaults to stdin if not set)", parse(from_os_str))]
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

fn progress_fn(progress: Progress) {
    match progress {
        Progress::Packet(p) => print!("."),
        _ => print!("\nProgress: {:?}", progress),
    }
}

fn main() {
    use std::fs::File;
    use std::io::{self, BufReader, BufRead};

    let opt = Opt::from_args();
    let mut port = serial::open(&opt.tty_path).expect("path points to invalid TTY");

    port.set_timeout(Duration::from_secs(opt.timeout)).expect("error setting timeout");
    let mut settings = port.read_settings().expect("error reading settings");
    settings.set_baud_rate(opt.baud_rate).unwrap();
    settings.set_stop_bits(opt.stop_bits);
    settings.set_flow_control(opt.flow_control);
    settings.set_char_size(opt.char_width);
    port.write_settings(&settings).expect("error writting settings");

    let mut handle: Box<dyn BufRead> =  match opt.input {
        Some(p) => Box::new(BufReader::new(File::open(&p).unwrap())),
        None => Box::new(BufReader::new(io::stdin())),
    };

    if opt.raw {
        io::copy(&mut *handle, &mut port).expect("raw copy failed");
    } else {
        Xmodem::transmit_with_progress(handle, &mut port, progress_fn).expect("xmodem transmit failed");
    }
}

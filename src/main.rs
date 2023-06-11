use clap::{Parser, ValueEnum};
use regex::Regex;
use std::error::Error;
use std::io::{self, prelude::*, Read};
use std::panic::panic_any;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[derive(Parser)]
#[command(version)]
struct Args {
    /// Device path to a serial port
    #[arg(long)]
    port: String,

    /// Output format
    #[arg(long, default_value_t = OutputFormat::KV)]
    format: OutputFormat,

    /// Process the output from a serial port only once and then exit
    #[arg(long, default_value_t = false)]
    once: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    JSON,
    KV,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            OutputFormat::JSON => write!(f, "json"),
            OutputFormat::KV => write!(f, "kv"),
        }
    }
}

enum COMMAND {
    STA,
    STP,
}

impl COMMAND {
    fn as_bytes(&self) -> &'static [u8] {
        match self {
            COMMAND::STA => "STA\r\n".as_bytes(),
            COMMAND::STP => "STP\r\n".as_bytes(),
        }
    }
}

struct UDCO2S {
    co2: String,
    hum: String,
    tmp: String,
}

impl UDCO2S {
    fn new(co2: &str, hum: &str, tmp: &str) -> UDCO2S {
        UDCO2S {
            co2: String::from(co2),
            hum: String::from(hum),
            tmp: String::from(tmp),
        }
    }

    /// OutputFormat::JSON
    /// => {"CO2":637,"HUM":56.5,"TMP":29.7}
    ///
    /// OutputFormat::KV
    /// => CO2=637,HUM=56.5,TMP=29.7
    fn format(self, output_format: OutputFormat) -> String {
        if output_format == OutputFormat::JSON {
            format!(
                "{{\"CO2\":{},\"HUM\":{},\"TMP\":{}}}",
                self.co2, self.hum, self.tmp
            )
        } else {
            format!("CO2={},HUM={},TMP={}", self.co2, self.hum, self.tmp)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let port_name = args.port;
    let output_format = args.format;
    let once = args.once;

    let mut is_first_detected = false;
    let mut serial_buf: Vec<u8> = vec![0; 1000];

    // CO2=1230,HUM=56.5,TMP=20.0
    let re = Regex::new(r"^CO2=(\d+),HUM=(\d+.\d+),TMP=(\d+.\d+)")?;

    let mut port = serialport::new(port_name, 115200)
        .timeout(Duration::from_millis(1000 * 10))
        .open()?;

    port.write(COMMAND::STA.as_bytes())?;
    std::io::stdout().flush()?;

    let chan_clear_buf = input_service();

    loop {
        match chan_clear_buf.try_recv() {
            Ok(_) => {}
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                break;
            }
        }

        let t = port.read(serial_buf.as_mut_slice())?;
        let line = std::str::from_utf8(&serial_buf[..t])?;
        let caps = re.captures(line);

        match caps {
            Some(v) => {
                let co2 = &v[1];
                let hum = &v[2];
                let tmp = &v[3];

                let udco2s = UDCO2S::new(co2, hum, tmp);
                println!("{}", udco2s.format(output_format));

                is_first_detected = true;
            }
            None => {
                // noop
            }
        }

        if once && is_first_detected {
            break;
        }
    }

    port.write(COMMAND::STP.as_bytes())?;
    std::mem::drop(port);
    std::io::stdout().flush()?;

    Ok(())
}

fn input_service() -> mpsc::Receiver<()> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut buffer = [0; 32];
        loop {
            match io::stdin().read(&mut buffer) {
                Ok(0) => {
                    // EOF
                    drop(tx);
                    break;
                }
                Ok(_) => {}
                Err(e) => panic_any(e),
            }
        }
    });

    rx
}

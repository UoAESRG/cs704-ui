
#[macro_use] extern crate log;
extern crate simplelog;
use simplelog::{SimpleLogger, LevelFilter};

extern crate structopt;
use structopt::StructOpt;

extern crate cs704_ui;
use cs704_ui::*;

#[derive(Clone, Debug, PartialEq, StructOpt)]
pub struct Options {
    #[structopt(short = "p", long = "port", default_value = "/dev/serial0")]
    /// Serial port for receiving NMEA data
    pub serial_port: String,

    #[structopt(short = "b", long = "baud", default_value = "115200")]
    /// Client ID for MQTT connection
    pub baud: usize,

    #[structopt(short = "z", long = "rezero")]
    /// Zero location based on first location packet
    pub rezero: bool,

    #[structopt(short = "i", long = "init-command")]
    /// Init command to be sent to the device
    pub init_command: Option<String>,

    #[structopt(short = "m", long = "mode-filter")]
    /// Location mode type to filter on
    pub mode_filter: Option<String>,

    #[structopt(long = "log-level", default_value = "info")]
    /// Enable verbose logging
    pub level: LevelFilter,
}

fn main() {
    // Load options
    let opts = Options::from_args();

    // Setup logging
    SimpleLogger::init(opts.level, simplelog::Config::default()).unwrap();

    // Connect to serial port
    debug!("Opening serial port ({}, {} baud)", opts.serial_port, opts.baud);
    let mut c = Connector::new(&opts.serial_port, opts.baud).unwrap();

    debug!("Connected to serial port");

    // Send init command if set
    if let Some(command) = &opts.init_command {
        debug!("Sending command: {}", command);
        c.write(command).unwrap();
    }

    let mut zero_x = 0.0;
    let mut zero_y = 0.0;
    let mut zeroed = false;

    loop {
        // Poll for new messages
        let m = match c.poll().unwrap() {
            Some(m) => m,
            None => continue,
        };

        // Handle incoming messages
        match &m {
            Message::Message(DebugMessage{body, data}) => {
                info!("Debug: {}, ({:?})", body, data);
            },
            Message::Raw(raw) => {
                info!("Raw: {:?}", raw);
            },
            Message::Location(loc) => {

                // Filter by location mode if specified
                if let Some(filter) = &opts.mode_filter {
                    if filter != &loc.mode {
                        continue;
                    }
                }

                // Apply zeroing if requested
                if opts.rezero && !zeroed {
                    info!("Applying new zero position X: {:.2}, Y: {:.2}", loc.x, loc.y);
                    zero_x = loc.x;
                    zero_y = loc.y;
                    zeroed = true;
                }

                // Extract and display X, Y, Mode
                let (x, y, mode) = (loc.x - zero_x, loc.y - zero_y, &loc.mode);
                print!("\r X: {:.2} Y: {:.2} Mode: {}", x, y, mode);
            },
        }
    }

}

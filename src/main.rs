use clap::{command, Parser, Subcommand};
use mutil::MessageOptions;
use std::process;
mod mutil;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Midi channel number
    #[arg(short, long)]
    channel: Option<u8>,

    /// Midi device id
    #[arg(short, long)]
    device: Option<i32>,
}

#[derive(Subcommand)]
enum Commands {
    /// Get list of midi devices
    Devices,
    /// Get list of midi input devices
    InputDevices,
    /// Get list of midi output devices
    OutputDevices,
    /// Open stream of incoming midi messages
    Stream,
    /// Send midi note on message
    NoteOn {
        /// midi note number
        note: u8,
        /// midi velocity number
        velocity: Option<u8>,
    },
    /// Send midi note off message
    NoteOff {
        /// midi note number
        note: u8,
    },
    /// Send midi note on message followed by note off message
    Trig {
        /// midi note number
        note: u8,
        /// midi velocity number
        velocity: Option<u8>,
    },
}

fn print_devices(devices: &Vec<mutil::Device>) {
    println!("{}", serde_json::to_string(devices).unwrap())
}

fn execute() -> Result<(), portmidi::Error> {
    let cli = Cli::parse();
    let mutil = mutil::Mutil::new()?;

    let message_options = MessageOptions {
        device: cli.device,
        channel: cli.channel,
    };

    match &cli.command {
        Commands::Devices => print_devices(&mutil.devices(None)?),
        Commands::InputDevices => {
            print_devices(&mutil.devices(Some(mutil::DeviceDirection::Input))?)
        }
        Commands::OutputDevices => {
            print_devices(&mutil.devices(Some(mutil::DeviceDirection::Output))?)
        }
        Commands::NoteOn { note, velocity } => mutil.note_on(*note, *velocity, message_options)?,
        Commands::NoteOff { note } => mutil.note_off(*note, message_options)?,
        Commands::Trig { note, velocity } => mutil.trig(*note, *velocity, message_options)?,
        Commands::Stream => {
            let rx = mutil.stream(None)?;

            loop {
                let msg = rx.recv().unwrap();
                println!("{}", msg.json())
            }
        }
    };

    Ok(())
}

fn main() {
    let result = execute();
    if result.is_err() {
        let err = result.err();
        eprintln!("{:?}", err);
        process::exit(1);
    }
}

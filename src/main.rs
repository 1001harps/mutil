use clap::{command, Parser, Subcommand};
use mutil::MessageOptions;
mod mutil;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    channel: Option<u8>,

    #[arg(short, long)]
    device: Option<i32>,
}

#[derive(Subcommand)]
enum Commands {
    Devices,
    InputDevices,
    OutputDevices,
    Stream,
    NoteOn { note: u8, velocity: Option<u8> },
    NoteOff { note: u8 },
    Trig { note: u8, velocity: Option<u8> },
}

fn print_devices(devices: &Vec<mutil::Device>) {
    println!("{}", serde_json::to_string(devices).unwrap())
}

fn main() {
    let cli = Cli::parse();
    let mutil = mutil::Mutil::new();

    let message_options = MessageOptions {
        device: cli.device,
        channel: cli.channel,
    };

    match &cli.command {
        Commands::Devices => print_devices(&mutil.devices(None)),
        Commands::InputDevices => {
            print_devices(&mutil.devices(Some(mutil::DeviceDirection::Input)))
        }
        Commands::OutputDevices => {
            print_devices(&mutil.devices(Some(mutil::DeviceDirection::Output)))
        }
        Commands::NoteOn { note, velocity } => mutil.note_on(*note, *velocity, message_options),
        Commands::NoteOff { note } => mutil.note_off(*note, message_options),
        Commands::Trig { note, velocity } => mutil.trig(*note, *velocity, message_options),
        Commands::Stream => {
            let rx = mutil.stream(None);

            loop {
                let msg = rx.recv().unwrap();
                println!("{}", msg.json())
            }
        }
    };
}

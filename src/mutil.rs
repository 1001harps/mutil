use serde::Serialize;
use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc,
    },
    thread,
    time::Duration,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(PartialEq, Clone)]
pub enum DeviceDirection {
    Input,
    Output,
}

#[derive(Serialize)]
pub struct Device {
    id: i32,
    name: String,
    direction: DeviceDirection,
}

impl Device {
    pub fn new(di: portmidi::DeviceInfo) -> Device {
        let direction = match di.direction() {
            portmidi::Direction::Input => DeviceDirection::Input,
            _ => DeviceDirection::Output,
        };

        Device {
            id: di.id(),
            name: di.name().to_string(),
            direction,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MidiMessageType {
    NoteOn,
    NoteOff,
    Todo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MidiMessageJson {
    #[serde(rename = "type")]
    type_: MidiMessageType,
    channel: u8,
    note: u8,
    velocity: u8,
}

#[derive(Debug)]
pub struct MidiMessage {
    status: u8,
    data1: u8,
    data2: u8,
    data3: u8,
}

impl MidiMessage {
    pub fn note_on(channel: u8, note: u8, velocity: Option<u8>) -> MidiMessage {
        MidiMessage {
            status: 0x90 + channel,
            data1: note,
            data2: velocity.unwrap_or(100),
            data3: 0,
        }
    }

    pub fn note_off(channel: u8, note: u8) -> MidiMessage {
        MidiMessage {
            status: 0x80 + channel,
            data1: note,
            data2: 0,
            data3: 0,
        }
    }

    pub fn to_portmidi(&self) -> portmidi::MidiMessage {
        portmidi::MidiMessage {
            status: self.status,
            data1: self.data1,
            data2: self.data2,
            data3: self.data3,
        }
    }

    pub fn from_portmidi(message: portmidi::MidiMessage) -> MidiMessage {
        MidiMessage {
            status: message.status,
            data1: message.data1,
            data2: message.data2,
            data3: message.data3,
        }
    }

    pub fn json(&self) -> String {
        let status = self.status & 0xf0 >> 4;
        let channel = self.status & 0xf;

        let type_ = match status {
            8 => MidiMessageType::NoteOff,
            9 => MidiMessageType::NoteOn,
            _ => MidiMessageType::Todo,
        };

        let msg = MidiMessageJson {
            type_,
            channel,
            note: self.data1,
            velocity: self.data2,
        };

        serde_json::to_string(&msg).unwrap()
    }
}

pub struct MessageOptions {
    pub device: Option<i32>,
    pub channel: Option<u8>,
}

pub struct Mutil {
    context: Arc<portmidi::PortMidi>,
}

impl Mutil {
    pub fn new() -> Result<Mutil, portmidi::Error> {
        let pm_context = portmidi::PortMidi::new()?;
        let context = Arc::new(pm_context);
        Ok(Mutil { context })
    }

    pub fn devices(
        &self,
        direction: Option<DeviceDirection>,
    ) -> Result<Vec<Device>, portmidi::Error> {
        let devices = self.context.devices().unwrap().into_iter().map(Device::new);

        if direction.is_none() {
            return Ok(devices.collect());
        }

        let dir = direction.unwrap();
        Ok(devices.filter(|d| d.direction == dir).collect())
    }

    pub fn note_on(
        &self,
        note: u8,
        velocity: Option<u8>,
        options: MessageOptions,
    ) -> Result<(), portmidi::Error> {
        let device_id = options
            .device
            .unwrap_or(self.context.default_output_device_id().unwrap());

        let channel = options.channel.unwrap_or(0);

        let mut out_port = self
            .context
            .device(device_id)
            .and_then(|dev| self.context.output_port(dev, 1024))
            .unwrap();

        out_port.write_message(MidiMessage::note_on(channel, note, velocity).to_portmidi())
    }

    pub fn note_off(&self, note: u8, options: MessageOptions) -> Result<(), portmidi::Error> {
        let device_id = options
            .device
            .unwrap_or(self.context.default_output_device_id().unwrap());

        let channel = options.channel.unwrap_or(0);

        let mut out_port = self
            .context
            .device(device_id)
            .and_then(|dev| self.context.output_port(dev, 1024))
            .unwrap();

        out_port.write_message(MidiMessage::note_off(channel, note).to_portmidi())
    }

    pub fn trig(
        &self,
        note: u8,
        velocity: Option<u8>,
        options: MessageOptions,
    ) -> Result<(), portmidi::Error> {
        let device_id = options
            .device
            .unwrap_or(self.context.default_output_device_id().unwrap());

        let channel = options.channel.unwrap_or(0);

        let mut out_port = self
            .context
            .device(device_id)
            .and_then(|dev| self.context.output_port(dev, 1024))
            .unwrap();

        out_port.write_message(MidiMessage::note_on(channel, note, velocity).to_portmidi())?;

        thread::sleep(Duration::from_millis(40));

        out_port.write_message(MidiMessage::note_off(channel, note).to_portmidi())
    }

    pub fn stream(&self, input_id: Option<i32>) -> Result<Receiver<MidiMessage>, portmidi::Error> {
        let timeout = Duration::from_millis(10);
        const BUF_LEN: usize = 1024;

        let (tx_from_port, rx_from_port) = mpsc::channel();

        let input_id = input_id.unwrap_or(self.context.default_input_device_id().unwrap());

        let devices = self.context.devices()?;

        let input_device = devices
            .clone()
            .into_iter()
            .find(|d| d.id() == input_id)
            .unwrap();

        println!(
            "opening stream: dev: {}, {}",
            input_device,
            input_device.id()
        );

        let context = self.context.clone();

        thread::spawn(move || {
            println!("thread");

            let input_port = context.input_port(input_device, BUF_LEN).unwrap();

            loop {
                if let Ok(Some(events)) = input_port.read_n(BUF_LEN) {
                    for event in events {
                        let msg = MidiMessage::from_portmidi(event.message);
                        tx_from_port.send(msg).unwrap();
                    }
                }

                thread::sleep(timeout);
            }
        });

        Ok(rx_from_port)
    }
}

use std::cmp;
use std::collections::VecDeque;

use portaudio::PortAudio;
use portaudio::DeviceIndex;
use portaudio::DeviceInfo;
use serde_derive::{Serialize, Deserialize};

pub struct AJDeviceManager {
    pa: PortAudio,
    config: AJConfig
}

pub struct AJInputDevice {
    pub name: String,
    dev_index: DeviceIndex,
    channels: i32
}

pub struct AJOutputDevice {
    pub name: String,
    dev_index: DeviceIndex,
    channels: i32
}

#[derive(Serialize, Deserialize, Copy)]
pub struct AJConfig {
    pub sample_rate: f64,
    pub channels: i32,
    pub frames: u32,
    pub volume: f32
}

impl Clone for AJConfig {
    fn clone(&self) -> AJConfig {
        *self
    }
}

impl std::fmt::Display for AJInputDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "AJInputDevice(name={}, channels={})", self.name, self.channels)
    }
}

impl std::fmt::Display for AJOutputDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "AJOutputDevice(name={}, channels={})", self.name, self.channels)
    }
}

fn pa_devices<R>(pa: &PortAudio, filter: fn(device: &DeviceInfo) -> bool, mapper: fn (idx: DeviceIndex, device: &DeviceInfo) -> R) -> Vec<R>{
    let devices = pa.devices().expect("Failed to connect PortAudio");
    devices.filter_map(|f| {
        let (idx, info): (DeviceIndex, DeviceInfo) = f.expect("Failed to get device info.");
        if filter(&info) {
            Some(mapper(idx, &info))
        } else {
            None
        }
    }).collect()
}


const INTERLEAVED: bool = true;

impl AJDeviceManager {
    pub fn get_input_devices(&self) -> Vec<AJInputDevice> {
        pa_devices(
            &self.pa,
            |di: &DeviceInfo| {di.max_input_channels > 0},
            |idx: DeviceIndex, di: &DeviceInfo| {
                AJInputDevice {
                    name: String::from(di.name),
                    dev_index: idx,
                    channels: di.max_input_channels
                }
            }
        )
    }

    pub fn get_default_output_device(&self) -> AJOutputDevice {
        let idx = self.pa.default_output_device().expect("Default output device not found.");
        let info = self.pa.device_info(idx).expect("Failed to get device info.");
        AJOutputDevice{
            name: String::from(info.name),
            dev_index: idx,
            channels: info.max_output_channels
        }
    }
    
    pub fn get_output_devices(&self) -> Vec<AJOutputDevice> {
        pa_devices(
            &self.pa,
            |di: &DeviceInfo| {di.max_output_channels > 0},
            |idx: DeviceIndex, di: &DeviceInfo| {
                AJOutputDevice {
                    name: String::from(di.name),
                    dev_index: idx,
                    channels: di.max_output_channels
                }
            }
        )
    }
    
    pub fn connect(&self, input: &AJInputDevice, output: &AJOutputDevice) {
        let AJDeviceManager { config, pa } = self;
        let input_dev_info = pa.device_info(input.dev_index).expect("Failed to connect input device");
        let output_dev_info = pa.device_info(output.dev_index).expect("Failed to connect output device");
        let channels = cmp::min(input.channels, output.channels);
    
        let input_params = portaudio::StreamParameters::<f32>::new(input.dev_index, channels, INTERLEAVED, input_dev_info.default_low_input_latency);
        let output_params = portaudio::StreamParameters::<f32>::new(output.dev_index, channels, INTERLEAVED, output_dev_info.default_low_output_latency);
    
        pa.is_duplex_format_supported(input_params, output_params, config.sample_rate).expect("Format not supported.");
    
        let settings = portaudio::DuplexStreamSettings::new(input_params, output_params, config.sample_rate, config.frames);
        let mut stream = pa.open_blocking_stream(settings).expect("Failed to open stream.");
    
        let mut buffer: VecDeque<f32> = VecDeque::with_capacity(config.frames as usize * config.channels as usize);
    
        fn wait_stream<F>(check_available: F) -> Result<u32, &'static str> 
            where 
                F: Fn () -> Result<portaudio::stream::Available, portaudio::Error>,
            {
            match check_available().expect("Stream input not available.") {
                portaudio::StreamAvailable::Frames(frames) => Result::Ok(frames as u32),
                portaudio::StreamAvailable::InputOverflowed => Result::Err("Input overflowed"),
                portaudio::StreamAvailable::OutputUnderflowed => Result::Err("Output underflowed")
            }
        }
    
        stream.start().expect("Failed to start stream.");
        'stream: loop {
            let in_frames = wait_stream(|| stream.read_available()).unwrap_or_default();
            if in_frames > 0 {
                match stream.read(in_frames) {
                    Ok(input_samples) =>
                        buffer.extend(input_samples.into_iter()),
                    Err(err) => eprintln!("E: Failed to read stream: {}", err)
                }
            }
            let out_frames = wait_stream(|| stream.write_available()).unwrap_or_default();
            let buffer_frames = (buffer.len() / config.channels as usize) as u32;
            let write_frames = cmp::min(out_frames, buffer_frames);
            // println!("IN {}", in_frames);
            // println!("OUT {}", write_frames);
            if write_frames > 0 {
                let n_write_samples = write_frames as usize * config.channels as usize;
                let res = stream.write(write_frames, |output| {
                    for i in 0..n_write_samples {
                        output[i] = config.volume * buffer.pop_front().expect("Buffer is empty.");
                    }
                });
                match res {
                    Err(e) => {
                        eprintln!("W: Failed to write streams: {}", e)
                    },
                    _ => {}
                }
            }
        }
    
    }
}

pub fn start(config: AJConfig) -> AJDeviceManager {
    let pa = portaudio::PortAudio::new().expect("Failed to connect PortAudio");
    AJDeviceManager{pa: pa, config: config}
}
use std::path::PathBuf;
use dirs::home_dir;
use super::device;
extern crate rprompt;
use toml;
use std::fs;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AJFileConfig {
    pub devices: AJDeviceConfig,
    pub sound: device::AJConfig
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AJDeviceConfig {
    pub input_device: String,
    pub output_device: String
}

fn config_file_path() -> PathBuf {
    let mut p = home_dir().expect("Can not get home dir");
    p.push(".audio-jack.toml");
    p
}

fn list_input_devices(manager: &device::AJDeviceManager){
    manager.get_input_devices().iter().enumerate().for_each(|(idx, d)| {
        println!("\t[{}] {}", idx, d.name);
    })
}

fn list_output_devices(manager: &device::AJDeviceManager){
    manager.get_output_devices().iter().enumerate().for_each(|(idx, d)| {
        println!("\t[{}] {}", idx, d.name);
    })
}

fn generate_config(volume: f32) -> device::AJConfig {
    device::AJConfig {
        channels: 2,
        frames: 256,
        sample_rate: 44_100.0,
        volume
    }
}

pub fn init_config(){
    let volume: f32 = rprompt::prompt_reply_stdout("Volume [0.0-1.0]: ")
        .expect("Failed to get volume.")
        .parse().expect("Not a number.");
    let conf = generate_config(volume);
    let manager = device::start(conf);
    let input_devices = manager.get_input_devices();
    let output_devices = manager.get_output_devices();
    println!("Select input device number:");
    input_devices.iter().enumerate().for_each(|(idx, d)| {
        println!("\t{}. {}", idx, d.name);
    });
    let input_dev_index: usize = rprompt::prompt_reply_stdout("Device Number: ")
        .expect("Failed to get device number.")
        .parse().expect("Not a number.");
    let input_dev_name = &input_devices[input_dev_index].name;
    println!("Select output device number:");
    output_devices.iter().enumerate().for_each(|(idx, d)| {
        println!("\t{}. {}", idx, d.name);
    });
    let output_dev_index: usize = rprompt::prompt_reply_stdout("Device Number: ")
        .expect("Failed to get device number.")
        .parse().expect("Not a number.");
    let output_dev_name = &output_devices[output_dev_index].name;
    let dev_conf = AJDeviceConfig {
        input_device: input_dev_name.clone(),
        output_device: output_dev_name.clone()
    };
    let result = AJFileConfig {
        devices: dev_conf,
        sound: conf
    };
    fs::write(config_file_path(), toml::to_string_pretty(&result).unwrap());
}

pub fn load_config() -> AJFileConfig {
    let bytes = fs::read(config_file_path()).expect("Failed to read config file. \nPlease run 'audio-jack init'.");
    let content = String::from_utf8(bytes).expect("");
    let res: AJFileConfig = toml::from_str(&content).expect("Invalid config file format.");
    res
}

pub fn clean_config(){
    fs::remove_file(config_file_path()).expect("Failed to remove config file.");
}
use ctrlc;
mod device;
mod cmd;
mod config;

fn connect_devices() {
    let conf = config::load_config();
    let manager = device::start(conf.sound);
    let input_devices = manager.get_input_devices();
    let output_devices = manager.get_output_devices();
    let input_device_name = &conf.devices.input_device;
    let output_device_name = &conf.devices.output_device;
    let in_dev = input_devices.iter().filter(|d| {d.name == *input_device_name}).next().expect("Input device not found.");
    let out_dev = output_devices.iter().filter(|d| {d.name == *output_device_name}).next().expect("Output device not found.");
    println!("{}", in_dev);
    println!("{}", out_dev);
    manager.connect(in_dev, out_dev);
}

fn main() {
    match cmd::parse_args() {
        cmd::CmdType::Init => config::init_config(),
        cmd::CmdType::Run => connect_devices(),
        cmd::CmdType::Clean => config::clean_config(),
        cmd::CmdType::Help => cmd::show_help_message(),
    };
}

use std::env::args;

pub enum CmdType {
    Init,
    Run,
    Clean,
    Help
}

pub fn show_help_message() {
    println!("audio-forward v0.1.0 JaccobKii<jaccobkii@gmail.com>");
    println!("Usage: audio-jack <command>");
    println!("\tinit\tInitialize configuration file.");
    println!("\trun\tConnect audio devices.");
    println!("\tclean\tRemove configuration file.");
    println!("\thelp\tShow this help message.")
}

pub fn parse_args() -> CmdType{
    let mut a = args();
    a.next().expect("Invalid arguments.");
    match a.next() {
        None => CmdType::Help,
        Some(s) => match s.as_str() {
            "i" | "init" => CmdType::Init,
            "r" | "run" => CmdType::Run,
            "c" | "clean" => CmdType::Clean,
            _ => CmdType::Help
        }
    }
}

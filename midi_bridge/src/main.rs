#![windows_subsystem = "windows"] // Delete this line if you dan't want the program to run in the background

// Libraries
use serialport::{available_ports, SerialPort};
use std::io::Read;
use std::time::{Duration, Instant};
use std::thread;
use midir::{MidiOutput};

// Autostart function
#[cfg(target_os = "windows")]
fn setup_autostart() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use winreg::enums::*;
    use winreg::RegKey;

    let exe_path = env::current_exe()?;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_WRITE,
    )?;

    run_key.set_value("MidiBridge", &exe_path.to_str().unwrap())?;
    println!("Autostart added succesfully: {}", exe_path.display());

    Ok(())
}

const ARDUINO_ID: [u8; 2] = [0xFA, 0xCE]; // Program will seek for this ID
const BAUD_RATE: u32 = 115_200; // Baud rate

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        if let Err(e) = setup_autostart() {
            eprintln!("Error during autostart setup: {}", e);
        }
    }

    println!("Arduino search starts...");

    loop {
        match find_and_connect_arduino() {
            Ok(_) => println!("Arduino disconnected, searching again..."),
            Err(e) => eprintln!("Error: {}, trying again after 2 seconds...", e),
        }
        thread::sleep(Duration::from_secs(2));
    }
}

fn find_and_connect_arduino() -> Result<(), Box<dyn std::error::Error>> {
    let ports = available_ports()?;

    for port_info in ports {
        println!("Port inspection: {}", port_info.port_name);
        if let Ok(mut port) = serialport::new(&port_info.port_name, BAUD_RATE)
            .timeout(Duration::from_millis(2000))
            .open()
        {
            let mut buffer = Vec::new();
            let mut temp_buf = [0u8; 1];

            let start = Instant::now();
            while start.elapsed() < Duration::from_secs(5) {
                match port.read(&mut temp_buf) {
                    Ok(1) => {
                        buffer.push(temp_buf[0]);
                        if buffer.ends_with(&ARDUINO_ID) {
                            println!("Arduino found: {}", port_info.port_name);
                            return midi_loop(port);
                        }
                        if buffer.len() > 10 {
                            buffer.drain(0..buffer.len()-2);
                        }
                    }
                    Ok(_) => continue,
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                    Err(e) => {
                        eprintln!("Port reading error: {}", e);
                        break;
                    }
                }
            }
            println!("Couldn't find Arduino ID in port {}", port_info.port_name);
        }
    }

    Err("Couldn't find Arduino".into())
}

fn midi_loop(mut serial: Box<dyn SerialPort>) -> Result<(), Box<dyn std::error::Error>> {
    let midi_out = MidiOutput::new("MIDI Bridge")?;
    let out_ports = midi_out.ports();

    println!("Available MIDI out ports:");
    for (i, port) in out_ports.iter().enumerate() {
        println!("Port {}: {}", i, midi_out.port_name(port)?);
    }

    // Lets find LoopMIDI by it's port name
    let loopmidi_port = out_ports.iter()
        .find(|p| midi_out.port_name(p).unwrap_or_default().contains("MIDI Bridge"))
        .ok_or("Couldn't find LoopMIDI port")?;

    let mut conn_out = midi_out.connect(loopmidi_port, "arduino-midi-out")?;
    println!("Connected to MIDI out port.");

    let mut buffer = [0u8; 3];

    loop {
        match serial.read_exact(&mut buffer) {
            Ok(_) => {
                conn_out.send(&buffer)?;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // No data, try again
                continue;
            }
            Err(e) => {
                eprintln!("Serial port disconnected: {}", e);
                break;
            }
        }
    }

    Ok(())
}

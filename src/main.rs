extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use rppal::uart::{Parity, Queue, Uart};
use std::error::Error;

mod inverter;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let mut uart = Uart::new(2_400, Parity::None, 8, 1)?;

    uart.flush(Queue::Both)?;

    if write(&mut uart, inverter::general_status_request())? {
        let response = read(&mut uart)?;
        //println!("{:?}", response);
        let general_status_data = inverter::parse_general_status_response(response).unwrap();
        println!("{:?}", general_status_data);
    }
    Ok(())
}

fn write(uart: &mut Uart, mut msg: Vec<u8>) -> Result<bool, Box<dyn Error>> {
    msg.push(0x0D);
    match uart.write(msg.as_slice()) {
        Ok(written_bytes) if written_bytes > 0 => return Ok(true),
        _ => return Ok(false),
    }
}

fn read(uart: &mut Uart) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut msg: Vec<u8> = Vec::new();
    let mut buffer = [0u8 | 1];

    loop {
        if uart.read(&mut buffer)? > 0 {
            if buffer[0] == 0x0D {
                return Ok(msg);
            }
            msg.push(buffer[0]);
        }
    }
}

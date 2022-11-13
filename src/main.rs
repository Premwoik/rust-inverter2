extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use rppal::i2c::I2c;
use rppal::uart::{Parity, Queue, Uart};
use std::error::Error;

use tokio::time::{sleep, Duration};

mod influxdb;
mod inverter;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    let client = influxdb::influx_new_client();
    let mut uart = Uart::new(2_400, Parity::None, 8, 1)?;
    tokio::spawn(read_counters());

    //loop {
        //uart.flush(Queue::Both)?;
        //if write(&mut uart, inverter::general_status_request())? {
            //let response = read(&mut uart)?;
            //match inverter::parse_general_status_response(response) {
                //Ok(general_status_data) => {
                    //let influx_msg = inverter::format_general_status(general_status_data);
                    //println!("{}", influx_msg);
                    //influxdb::write(&client, influx_msg);
                //}
                //Err(e) => println!("Error: {}\n", e),
            //}
        //}
        //sleep(Duration::from_secs(30)).await;
    //}
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

async fn read_counters() {
    let client = influxdb::influx_new_client();
    let mut i2c = I2c::new().unwrap();
    i2c.set_slave_address(8).unwrap();
    let mut buffer = [0u8; 5];
    loop {
        match i2c.read(&mut buffer) {
            Ok(_) => {
                let em = inverter::parse_energy_packet(&buffer);
                let msg = inverter::format_energy_meters(em);
                println!("{:?}\n", msg);
                influxdb::write(&client, msg);
            }
            Err(e) => println!("I2c read error {}\n", e),
        };
        sleep(Duration::from_secs(300)).await;
    }
}

use rppal::i2c::I2c;
use rppal::uart::{Parity, Queue, Uart};
use std::error::Error;

use tokio::time::{sleep, Duration};

mod influxdb;
mod inverter;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let client = influxdb::influx_new_client();
    let mut uart = Uart::new(2_400, Parity::None, 8, 1)?;
    tokio::spawn(read_counters());

    loop {
        uart.flush(Queue::Both)?;
        if write(&mut uart, inverter::general_status_request())? {
            let response = read(&mut uart).await?;
            match inverter::parse_general_status_response(response) {
                Ok(general_status_data) => {
                    let influx_msg = inverter::format_general_status(general_status_data);
                    println!("{}", influx_msg);
                    influxdb::write(&client, influx_msg);
                }
                Err(e) => println!("Error: {}\n", e),
            }
        }
        sleep(Duration::from_secs(30)).await;
    }
}

fn write(uart: &mut Uart, mut msg: Vec<u8>) -> Result<bool, Box<dyn Error>> {
    msg.push(0x0D);
    match uart.write(msg.as_slice()) {
        Ok(written_bytes) if written_bytes > 0 => return Ok(true),
        _ => return Ok(false),
    }
}

async fn read(uart: &mut Uart) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut msg: Vec<u8> = Vec::new();
    let mut buffer = [0u8 | 1];

    let mut counter = 0;
    while counter < 10 {
        if uart.read(&mut buffer)? > 0 {
            if buffer[0] == 0x0D {
                return Ok(msg);
            }
            msg.push(buffer[0]);
        } else {
            counter += 1;
            sleep(Duration::from_millis(200)).await;
        }
    }
    return Err("Timeouted while waiting for end byte".into());
}

async fn read_counters() {
    let client = influxdb::influx_new_client();
    let mut i2c = I2c::new().unwrap();
    i2c.set_slave_address(8).unwrap();
    loop {
        let succ = match try_read_measurements(&mut i2c) {
            Ok(msg) => {
                println!("{}\n", msg);
                influxdb::write(&client, msg);
                true
            }
            Err(e) => {
                println!("I2c read error {}\n", e);
                false
            }
        };

        if !succ {
            for _ in 1..5 {
                sleep(Duration::from_millis(100)).await;
                match try_read_old_measurements(&mut i2c) {
                    Ok(msg) => {
                        println!("1 - {}\n", msg);
                        influxdb::write(&client, msg);
                        break;
                    }
                    Err(e) => {
                        println!("1 - I2c read error {}\n", e);
                    }
                }
            }
        }

        sleep(Duration::from_secs(60)).await;
    }
}

fn try_read_measurements(i2c: &mut I2c) -> Result<String, Box<dyn Error>> {
    let mut buffer = [0u8; 10];
    i2c.read(&mut buffer)?;
    println!("{:?}", buffer);
    let object = inverter::parse_energy_packet(&buffer.to_vec())?;
    let msg = inverter::format_energy_meters(object);
    Ok(msg)
}

fn try_read_old_measurements(i2c: &mut I2c) -> Result<String, Box<dyn Error>> {
    let mut buffer = [0u8; 10];
    i2c.write_read(&[0x28, 0x01, 0x0D], &mut buffer)?;
    println!("{:?}", buffer);
    let object = inverter::parse_energy_packet(&buffer.to_vec())?;
    let msg = inverter::format_energy_meters(object);
    Ok(msg)
}

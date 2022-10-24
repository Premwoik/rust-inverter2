extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use std::fmt::Error;

mod inverter;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    pretty_env_logger::init();

    let val = inverter::read_general_status();
    println!("{:?}", val);

    Ok(())
}

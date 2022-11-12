const CMD_MODE_INQUIRY: &str = "QMOD";
const CMD_GENERAL_STATUS: &str = "QPIGS";
const CMD_RATING_INFORMATION: &str = "QPIRI";

use std::array::TryFromSliceError;
use std::error::Error;
use std::str::{self, FromStr};

#[allow(dead_code)]
#[derive(Debug)]
pub struct DeviceGeneralStatus {
    grid_voltage: f32,
    grid_frequency: f32,
    ac_output_voltage: f32,
    ac_output_frequency: f32,
    ac_output_apparent_power: u16,
    ac_output_active_power: u16,
    ac_output_load: u16,
    bus_voltage: u16,
    battery_voltage: f32,
    battery_charging_current: u16,
    battery_capacity: u16,
    inverter_heat_sink_temperature: u16,
    pv_input_current: f32,
    pv_input_voltage: f32,
    battery_voltage_scc: f32,
    battery_discharge_current: u16,
    device_status: [u8; 8],
}

pub fn general_status_m(d: DeviceGeneralStatus) -> String {
    return format!("inverter_general_status,inverter_id=1 grid_voltage={},grid_freq={},ac_output_voltage={},ac_output_freq={},ac_output_apparent_power={},ac_output_active_power={},ac_output_load={},bus_voltage={},battery_voltage={},battery_charging_current={},battery_capacity={},inverter_temp={},pv_input_current={},pv_input_voltage={},pv_input_power={:.2},battery_voltage_scc={},battery_discharge_current={}\n",
                   d.grid_voltage,
                   d.grid_frequency,
                   d.ac_output_voltage,
                   d.ac_output_frequency,
                   d.ac_output_apparent_power,
                   d.ac_output_active_power,
                   d.ac_output_load,
                   d.bus_voltage,
                   d.battery_voltage,
                   d.battery_charging_current,
                   d.battery_capacity,
                   d.inverter_heat_sink_temperature,
                   d.pv_input_current,
                   d.pv_input_voltage,
                   d.pv_input_voltage * d.pv_input_current,
                   d.battery_voltage_scc,
                   d.battery_discharge_current
                   );
}

pub fn general_status_request() -> Vec<u8> {
    let mut request = convert_cmd(CMD_GENERAL_STATUS);
    append_crc(&mut request);
    return request;
}

#[allow(dead_code)]
pub fn mode_inquiry_request() -> Vec<u8> {
    let mut request = convert_cmd(CMD_MODE_INQUIRY);
    append_crc(&mut request);
    return request;
}

#[allow(dead_code)]
pub fn rating_information_request() -> Vec<u8> {
    let mut request = convert_cmd(CMD_RATING_INFORMATION);
    append_crc(&mut request);
    return request;
}

pub fn parse_general_status_response(
    response: Vec<u8>,
) -> Result<DeviceGeneralStatus, Box<dyn Error>> {
    if validate_crc(&response) {
        // Truncate the CRC
        let mut response2 = response.to_owned();
        response2.truncate(response2.len().saturating_sub(2));
        return parse_general_status(response2);
    } else {
        return Err("Invlid CRC".into());
    }
}

#[allow(dead_code)]
pub fn parse_mode_inquiry_response(response: Vec<u8>) -> Option<&'static str> {
    if validate_crc(&response) {
        return Some("Crc ok");
    } else {
        return None;
    }
}

#[allow(dead_code)]
pub fn parse_rating_information_response(response: Vec<u8>) -> Option<&'static str> {
    if validate_crc(&response) {
        return Some("Crc ok");
    } else {
        return None;
    }
}

fn parse_general_status(result: Vec<u8>) -> Result<DeviceGeneralStatus, Box<dyn Error>> {
    let mut offset: u8 = 1; // Skip start byte '('
    return Ok(DeviceGeneralStatus {
        grid_voltage: convert(&result, &mut offset, 5)?,
        grid_frequency: convert(&result, &mut offset, 4)?,
        ac_output_voltage: convert(&result, &mut offset, 5)?,
        ac_output_frequency: convert(&result, &mut offset, 4)?,
        ac_output_apparent_power: convert(&result, &mut offset, 4)?,
        ac_output_active_power: convert(&result, &mut offset, 4)?,
        ac_output_load: convert(&result, &mut offset, 3)?,
        bus_voltage: convert(&result, &mut offset, 3)?,
        battery_voltage: convert(&result, &mut offset, 5)?,
        battery_charging_current: convert(&result, &mut offset, 3)?,
        battery_capacity: convert(&result, &mut offset, 3)?,
        inverter_heat_sink_temperature: convert(&result, &mut offset, 4)?,
        pv_input_current: convert(&result, &mut offset, 4)?,
        pv_input_voltage: convert(&result, &mut offset, 5)?,
        battery_voltage_scc: convert(&result, &mut offset, 5)?,
        battery_discharge_current: convert(&result, &mut offset, 5)?,
        device_status: convert_device_status(&result, offset)?,
    });
}

fn convert_device_status(result: &Vec<u8>, offset: u8) -> Result<[u8; 8], TryFromSliceError> {
    let start = offset as usize;
    let end = (offset + 8) as usize;
    return result[start..end].try_into();
}

fn convert<T: FromStr>(
    result: &Vec<u8>,
    offset: &mut u8,
    size: u8,
) -> Result<T, Box<dyn Error + 'static>>
where
    <T as FromStr>::Err: Error + 'static,
{
    let start = *offset as usize;
    let end = (*offset + size) as usize;

    *offset = *offset + size + 1;

    let as_str = str::from_utf8(&result[start..end])?;

    return Ok(as_str.parse::<T>()?);
}

fn convert_cmd(cmd: &str) -> Vec<u8> {
    return cmd.chars().map(|c| c as u8).collect();
}

// https://www.nongnu.org/avr-libc/user-manual/group__util__crc.html
fn calculate_crc(data: &Vec<u8>) -> u16 {
    return data.iter().fold(0 as u16, |crc, &v| {
        let mut crc = crc ^ ((v as u16) << 8);
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = crc << 1 ^ 0x1021;
            } else {
                crc = crc << 1
            }
        }
        return crc;
    });
}

fn append_crc(data: &mut Vec<u8>) -> () {
    let crc = calculate_crc(data);
    data.push(high_crc(crc));
    data.push(low_crc(crc));
}

fn validate_crc(data: &Vec<u8>) -> bool {
    if data.len() < 3 {
        return false;
    }
    let len = data.len();
    let crc = calculate_crc(&(&data[..len - 2]).to_vec());
    return high_crc(crc) == data[len - 2] && low_crc(crc) == data[len - 1];
}

#[inline(always)]
fn high_crc(crc: u16) -> u8 {
    return (((crc) >> 8) & 0xFF) as u8;
}

#[inline(always)]
fn low_crc(crc: u16) -> u8 {
    return ((crc) & 0xFF) as u8;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_calculate_crc_correctly() {
        let cmd1 = convert_cmd(CMD_GENERAL_STATUS);
        assert_eq!(calculate_crc(&cmd1), 47017, "crc1 doesn't match");
        let cmd2 = convert_cmd(CMD_MODE_INQUIRY);
        assert_eq!(calculate_crc(&cmd2), 18881, "crc2 doesn't match");
        let cmd3 = convert_cmd(CMD_RATING_INFORMATION);
        assert_eq!(calculate_crc(&cmd3), 63572, "crc3 doesn't match");
    }

    #[test]
    fn can_split_crc_into_bytes() {
        let crc1 = 47017 as u16;
        assert_eq!(high_crc(crc1), 0xB7, "crc1 higher byte is incorrect");
        assert_eq!(low_crc(crc1), 0xA9, "crc1 lower byte iis incorrect");

        let crc2 = 18881 as u16;
        assert_eq!(high_crc(crc2), 0x49, "crc2 higher byte is incorrect");
        assert_eq!(low_crc(crc2), 0xC1, "crc2 lower byte iis incorrect");

        let crc3 = 63572 as u16;
        assert_eq!(high_crc(crc3), 0xF8, "crc3 higher byte is incorrect");
        assert_eq!(low_crc(crc3), 0x54, "crc3 lower byte iis incorrect");
    }

    #[test]
    fn can_append_crc_correctly() {
        let mut cmd = convert_cmd(CMD_GENERAL_STATUS);
        append_crc(&mut cmd);
        assert_eq!(
            cmd,
            [81, 80, 73, 71, 83, 0xB7, 0xA9],
            "wrong resulting vector"
        );
    }

    #[test]
    fn can_parse_general_status_response_correctly() {
        let init_char: Vec<u8> = vec![0x28];
        let grid_voltage = vec![0x30, 0x35, 0x31, 0x2E, 0x32, 0x20];
        let grid_frequency = vec![0x35, 0x30, 0x2E, 0x30, 0x20];
        let ac_output_voltage = vec![0x33, 0x30, 0x31, 0x2E, 0x32, 0x20];
        let ac_output_frequency = vec![0x35, 0x30, 0x2E, 0x30, 0x20];
        let ac_output_apparent_power = vec![0x32, 0x30, 0x31, 0x30, 0x20];
        let ac_output_active_power = vec![0x31, 0x39, 0x32, 0x30, 0x20];
        let ac_output_load = vec![0x30, 0x32, 0x35, 0x20];
        let bus_voltage = vec![0x30, 0x32, 0x30, 0x20];
        let battery_voltage = vec![0x35, 0x32, 0x2E, 0x31, 0x30, 0x20];
        let battery_charging_current = vec![0x30, 0x35, 0x34, 0x20];
        let battery_capacity = vec![0x31, 0x30, 0x30, 0x20];
        let inverter_heat_sink_temperature = vec![0x30, 0x30, 0x39, 0x30, 0x20];
        let pv_input_current = vec![0x34, 0x30, 0x2E, 0x30, 0x20];
        let pv_input_voltage = vec![0x30, 0x34, 0x30, 0x2E, 0x35, 0x20];
        let battery_voltage_scc = vec![0x35, 0x30, 0x2E, 0x32, 0x35, 0x20];
        let battery_discharge_current = vec![0x30, 0x30, 0x30, 0x32, 0x35, 0x20];
        let device_status = vec![0xB7, 0xB6, 0xB5, 0xB4, 0xB3, 0xB2, 0xB1, 0xB0, 0x20];
        let response: Vec<u8> = vec![
            init_char,
            grid_voltage,
            grid_frequency,
            ac_output_voltage,
            ac_output_frequency,
            ac_output_apparent_power,
            ac_output_active_power,
            ac_output_load,
            bus_voltage,
            battery_voltage,
            battery_charging_current,
            battery_capacity,
            inverter_heat_sink_temperature,
            pv_input_current,
            pv_input_voltage,
            battery_voltage_scc,
            battery_discharge_current,
            device_status,
        ]
        .into_iter()
        .flatten()
        .collect();

        let ready = parse_general_status(response).unwrap();
        assert_eq!(ready.grid_voltage, 51.2, "wrong grid_voltage");
        assert_eq!(ready.grid_frequency, 50.0, "wrong grid_frequency");
        assert_eq!(ready.ac_output_voltage, 301.2, "wrong ac_output_voltage");
        assert_eq!(ready.ac_output_frequency, 50.0, "wrong ac_output_frequency");
        assert_eq!(
            ready.ac_output_apparent_power, 2010,
            "wrong ac_output_apparent_power"
        );
        assert_eq!(
            ready.ac_output_active_power, 1920,
            "wrong ac_output_active_power"
        );
        assert_eq!(ready.ac_output_load, 25, "wrong ac_output_load");
        assert_eq!(ready.bus_voltage, 20, "wrong bus_voltage");
        assert_eq!(ready.battery_voltage, 52.10, "wrong battery_voltage");
        assert_eq!(
            ready.battery_charging_current, 54,
            "wrong battery_charging_current"
        );
        assert_eq!(ready.battery_capacity, 100, "wrong battery_capacity");
        assert_eq!(
            ready.inverter_heat_sink_temperature, 90,
            "wrong inverter_heat_sink_temperature"
        );
        assert_eq!(ready.pv_input_current, 40.0, "wrong pv_input_current");
        assert_eq!(ready.pv_input_voltage, 40.5, "wrong pv_input_voltage");
        assert_eq!(
            ready.battery_voltage_scc, 50.25,
            "wrong battery_voltage_scc"
        );
        assert_eq!(
            ready.battery_discharge_current, 25,
            "wrong battery_discharge_current"
        );
        assert_eq!(
            ready.device_status,
            [0xB7, 0xB6, 0xB5, 0xB4, 0xB3, 0xB2, 0xB1, 0xB0],
            "wrong device_status"
        );
    }
}

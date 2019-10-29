
use std::io::{Write, BufRead, BufReader, Error as IoError};
use std::collections::HashMap;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde;
extern crate serde_json;

extern crate serial;
use serial::prelude::*;
use serial::SystemPort;

pub struct Connector {
    port: BufReader<SystemPort>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "LOC")]
    Location(Location),

    #[serde(rename = "RAW")]
    Raw(Raw),

    #[serde(rename = "MSG")]
    Message(DebugMessage),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Location {
    pub x: f32,
    pub y: f32,
    pub h: Option<f32>,
    pub mode: String,
    pub update_rate: usize,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Raw {
    pub accel_x: f32,
    pub accel_y: f32,
    pub accel_z: f32,

    pub gyro_x: f32,
    pub gyro_y: f32,
    pub gyro_z: f32,

    pub mag_x: f32,
    pub mag_y: f32,
    pub mag_z: f32,

    pub sampling_rate: usize,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DebugMessage {
    pub body: String,
    pub data: HashMap<String, String>,
}

impl Connector {
    pub fn new(port: &str, baud: usize) -> Result<Self, IoError> {
        let p = std::path::Path::new(port);

        let mut port = serial::open(p)?;

        port.reconfigure(&|settings| {
            settings.set_baud_rate(serial::BaudRate::from_speed(baud))?;
            Ok(())
        })?;

        Ok(Self{port: BufReader::new(port)})
    }
 
    pub fn poll(&mut self) -> Result<Option<Message>, IoError> {

        // Read line from serial port
        let mut s = String::default();
        let _n = self.port.read_line(&mut s)?;

        debug!("read line: {}", s);

        // Attempt to decode
        match serde_json::from_str(&s) {
            Ok(v) => return Ok(Some(v)),
            Err(e) => {
                debug!("Error decoding line: {:?}", e);
                debug!("{}", s);
                return Ok(None)
            }
        }
    }

    pub fn write(&mut self, s: &str) -> Result<(), IoError> {
        let p = self.port.get_mut();

        p.write(s.as_bytes())?;

        Ok(())
    }
}



#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_decode_location() {
        let s = r#"{ "type": "LOC", "x": 11.22, "y": 33.44, "h":1.57, "mode": "IMU", "update_rate": 100 }"#;
        
        let m: Message = serde_json::from_str(s).unwrap();

        if let Message::Location(loc) = &m {

        } else {
            assert_eq!(1, 2);
        }
    }

    #[test]
    fn test_decode_debug_message() {
        let s = r#"{ "type": "RAW","accel_x": 11.22, "accel_y": 33.44, "accel_z": 55.66, "gyro_r": 11.22, "gyro_y": 33.44, "gyro_z": 55.66, "mag_x": 11.22, "mag_y": 33.44, "mag": 55.66,"sampling_rate": 100 }"#;
        
        let m: Message = serde_json::from_str(s).unwrap();

        if let Message::Raw(raw) = &m {

        } else {
            assert_eq!(1, 2);
        }
    }

}

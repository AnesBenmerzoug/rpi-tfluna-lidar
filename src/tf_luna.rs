mod constants;

use rppal::i2c::{Error, I2c};

#[derive(Debug)]
pub struct TFLuna {
    i2c: I2c,
}

#[derive(Clone, Debug)]
pub struct TFLunaInformation {
    firmware_version: String,
    serial_number: String,
    frame_rate: u16,
    slave_address: u8,
    mode: RangingMode,
    enable: bool,
    power_mode: PowerMode,
    signal_strength_threshold: u16,
    dummy_distance: u16,
    minimum_distance: u16,
    maximum_distance: u16,
}

#[derive(Clone, Copy, Debug)]
pub enum RangingMode {
    Continuous,
    Trigger,
}

#[derive(Clone, Copy, Debug)]
pub enum PowerMode {
    Normal,
    LowPower,
}

#[derive(Clone, Copy, Debug)]
pub struct SensorReading {
    pub distance: u16,
    pub signal_strength: u16,
    pub temperature: f32,
    pub timestamp: u16,
}

impl TFLuna {
    pub fn new(i2c: I2c) -> Result<Self, Error> {
        Ok(Self { i2c })
    }

    // Set enable bit
    pub fn enable(&mut self) -> Result<(), Error> {
        self.i2c.write(&[0x25, 0x01])?;
        Ok(())
    }

    // Unset enable bit
    pub fn disable(&mut self) -> Result<(), Error> {
        self.i2c.write(&[0x25, 0x00])?;
        Ok(())
    }

    // Reads distance, signal strength, temperature and timestamp
    pub fn read(&mut self) -> Result<SensorReading, Error> {
        let distance = self.read_two_byte_value(constants::DISTANCE_REGISTER_ADDRESS)?;
        let signal_strength =
            self.read_two_byte_value(constants::SIGNAL_STRENGTH_REGISTER_ADDRESS)?;
        let temperature =
            self.read_two_byte_value(constants::TEMPERATURE_REGISTER_ADDRESS)? as f32 / 100.0;
        let timestamp = self.read_two_byte_value(constants::TIMESTAMP_REGISTER_ADDRESS)?;
        Ok(SensorReading {
            distance,
            signal_strength,
            temperature,
            timestamp,
        })
    }

    // Read the contents of a single register
    fn read_register(&mut self, register_address: u8) -> Result<u8, Error> {
        // Send register address first
        self.i2c.write(&[register_address])?;
        // Read content of register
        let mut buffer = [0];
        self.i2c.read(&mut buffer)?;
        Ok(buffer[0])
    }

    // Read a value whose lower byte is at start_addres
    // and whose upper byte is at start_address + 1
    fn read_two_byte_value(&mut self, start_address: u8) -> Result<u16, Error> {
        let mut buffer = [0; 2];
        for i in 0..=1 {
            buffer[i] = self.read_register(start_address + i as u8)?;
        }
        let value = buffer[0] as u16 + ((buffer[1] as u16) << 8);
        Ok(value)
    }

    pub fn get_firmware_version(&mut self) -> Result<String, Error> {
        let mut buffer = [0; 3];
        for i in 0..=2 {
            buffer[i] = self.read_register(0x0A + i as u8)?;
        }
        let version = format!("{}.{}.{}", buffer[2], buffer[1], buffer[0]);
        Ok(version)
    }

    pub fn get_serial_number(&mut self) -> Result<String, Error> {
        let mut buffer = [0; 14];
        for i in 0..14 {
            buffer[i] = self.read_register(0x10 + i as u8)?;
        }
        let serial_number = buffer
            .into_iter()
            .map(|x| format!("{}", x))
            .reduce(|acc, e| {
                let mut acc = acc;
                acc.push_str(&e);
                acc
            })
            // TODO: error handling
            .unwrap();
        Ok(serial_number)
    }

    // Prints the content of all registers
    pub fn show_raw_register_contents(&mut self) -> Result<(), Error> {
        println!("Debug: Showing raw register contents");
        for addr in 0..0x3F {
            let value = self.read_register(addr)?;
            println!("register address = 0x{:x}, content = 0x{:x}", addr, value);
        }
        Ok(())
    }

    fn get_frame_rate(&mut self) -> Result<u16, Error> {
        self.read_two_byte_value(constants::FRAMERATE_REGISTER_ADDRESS)
    }

    fn get_slave_address(&mut self) -> Result<u8, Error> {
        self.read_register(constants::SLAVE_ADDRESS_REGISTER_ADDRESS)
    }

    fn get_ranging_mode(&mut self) -> Result<RangingMode, Error> {
        match self.read_register(constants::RANGING_MODE_REGISTER_ADDRESS)? {
            0 => Ok(RangingMode::Continuous),
            1 => Ok(RangingMode::Trigger),
            _ => Err(Error::FeatureNotSupported),
        }
    }
    fn get_enable(&mut self) -> Result<bool, Error> {
        match self.read_register(constants::ENABLE_REGISTER_ADDRESS)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::FeatureNotSupported),
        }
    }

    fn get_power_mode(&mut self) -> Result<PowerMode, Error> {
        match self.read_register(constants::POWER_MODE_REGISTER_ADDRESS)? {
            0 => Ok(PowerMode::Normal),
            1 => Ok(PowerMode::LowPower),
            _ => Err(Error::FeatureNotSupported),
        }
    }

    fn get_signal_strength_threshold(&mut self) -> Result<u16, Error> {
        self.read_two_byte_value(constants::SIGNAL_STRENGTH_THRESHOLD_REGISTER_ADDRESS)
    }

    fn get_dummy_distance(&mut self) -> Result<u16, Error> {
        self.read_two_byte_value(constants::DUMMY_DISTANCE_REGISTER_ADDRESS)
    }

    fn get_minimum_distance(&mut self) -> Result<u16, Error> {
        self.read_two_byte_value(constants::MINIMUM_DISTANCE_REGISTER_ADDRESS)
    }

    fn get_maximum_distance(&mut self) -> Result<u16, Error> {
        self.read_two_byte_value(constants::MAXIMUM_DISTANCE_REGISTER_ADDRESS)
    }

    // Print important information
    pub fn get_device_information(&mut self) -> Result<TFLunaInformation, Error> {
        let firmware_version = self.get_firmware_version()?;
        let serial_number = self.get_serial_number()?;
        let frame_rate = self.get_frame_rate()?;
        let slave_address = self.get_slave_address()?;
        let mode = self.get_ranging_mode()?;
        let enable = self.get_enable()?;
        let power_mode = self.get_power_mode()?;
        let signal_strength_threshold = self.get_signal_strength_threshold()?;
        let dummy_distance = self.get_dummy_distance()?;
        let minimum_distance = self.get_minimum_distance()?;
        let maximum_distance = self.get_maximum_distance()?;
        Ok(TFLunaInformation {
            firmware_version,
            serial_number,
            frame_rate,
            slave_address,
            mode,
            enable,
            power_mode,
            signal_strength_threshold,
            dummy_distance,
            minimum_distance,
            maximum_distance,
        })
    }
}

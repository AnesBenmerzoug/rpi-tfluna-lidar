use std::cell::RefCell;
use std::rc::Rc;

use embedded_hal::i2c::I2c as I2cTrait;
use pwm_pca9685::{Channel, Pca9685};

// Servo configuration.
// -45 degrees
const FIRST_ANGLE: f32 = -45.0;
const FIRST_ANGLE_COUNTER: u32 = 200;
/// 45 degrees
const SECOND_ANGLE: f32 = 45.0;
const SECOND_ANGLE_COUNTER: u32 = 410;
/// Slope
const SLOPE: f32 =
    ((SECOND_ANGLE_COUNTER - FIRST_ANGLE_COUNTER) as f32) / (SECOND_ANGLE - FIRST_ANGLE);
/// Intercept
const INTERCEPT: f32 = (SECOND_ANGLE_COUNTER as f32) - SLOPE * SECOND_ANGLE;

#[derive(Debug, Clone)]
pub enum Error {
    InvalidParameter(String),
    Other(String),
}

pub struct ServoMotor<I2c: I2cTrait> {
    pwm: Rc<RefCell<Pca9685<I2c>>>,
    channel: Channel,
    min_angle: f32,
    max_angle: f32,
    reversed: bool,
}

impl<I2c: I2cTrait> ServoMotor<I2c> {
    pub fn new(
        pwm: Rc<RefCell<Pca9685<I2c>>>,
        channel: Channel,
        min_angle: f32,
        max_angle: f32,
        reversed: bool,
    ) -> Result<ServoMotor<I2c>, Error> {
        let mut servo: ServoMotor<I2c> = ServoMotor {
            pwm,
            channel,
            min_angle,
            max_angle,
            reversed,
        };
        servo.set_angle(0.0)?;
        Ok(servo)
    }

    pub fn set_angle(&mut self, angle: f32) -> Result<(), Error> {
        if !self.is_angle_allowed(angle) {
            Err(Error::InvalidParameter(format!(
                "Provided angle '{}' is outside of valid range [{}, {}]",
                angle, self.min_angle, self.max_angle,
            )))
        } else {
            let pulse = match self.reversed {
                true => -SLOPE * angle + INTERCEPT,
                false => SLOPE * angle + INTERCEPT,
            };
            self.pwm
                .borrow_mut()
                .set_channel_on(self.channel, 0)
                .map_err(|_x| Error::Other(format!("Failed setting pulse width: {pulse}")))?;
            self.pwm
                .borrow_mut()
                .set_channel_off(self.channel, pulse as u16)
                .map_err(|_x| Error::Other(format!("Failed setting pulse width: {pulse}")))?;
            Ok(())
        }
    }

    pub fn get_min_angle(&self) -> f32 {
        self.min_angle
    }

    pub fn get_max_angle(&self) -> f32 {
        self.max_angle
    }

    pub fn is_angle_allowed(&self, angle: f32) -> bool {
        (angle >= self.min_angle) && (angle <= self.max_angle)
    }
}

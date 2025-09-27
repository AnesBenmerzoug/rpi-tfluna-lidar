use std::cell::RefCell;
use std::rc::Rc;

use embedded_hal::i2c::I2c as I2cTrait;
use pwm_pca9685::{Channel, Pca9685};

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
    intercept: f32,
    slope: f32,
}

impl<I2c: I2cTrait> ServoMotor<I2c> {
    pub fn new(
        pwm: Rc<RefCell<Pca9685<I2c>>>,
        channel: Channel,
        min_angle_counter: u32,
        max_angle_counter: u32,
        min_angle: f32,
        max_angle: f32,
    ) -> Result<ServoMotor<I2c>, Error> {
        let slope = ((max_angle_counter - min_angle_counter) as f32) / (max_angle - min_angle);
        let intercept = max_angle_counter as f32 - slope * max_angle;
        let mut servo: ServoMotor<I2c> = ServoMotor {
            pwm,
            channel,
            min_angle,
            max_angle,
            intercept,
            slope,
        };
        servo.set_angle(0.0)?;
        Ok(servo)
    }

    pub fn set_angle(&mut self, angle: f32) -> Result<(), Error> {
        if (angle > self.max_angle) || (angle < self.min_angle) {
            Err(Error::InvalidParameter(format!(
                "Provided angle '{}' is outside of valid range [{}, {}]",
                angle, self.min_angle, self.max_angle,
            )))
        } else {
            let pulse = self.slope * angle + self.intercept;
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
}

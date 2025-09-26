use std::time::Duration;
use std::error::Error;

use rppal::pwm::{Pwm};

pub struct ServoMotor {
    pwm: Pwm,
    max_angle: u64,
    intercept: f64,
    slope: f64,
}

impl ServoMotor {
    pub fn new(
        pwm: Pwm,
        neutral_pulse: u64,
        max_pulse: u64,
        max_angle: u64,
    ) -> Result<ServoMotor, Box<dyn Error>> {
        let intercept = neutral_pulse as f64;
        let slope = (max_pulse as f64 - intercept) / (max_angle as f64);
        let servo: ServoMotor = ServoMotor {
            pwm,
            max_angle,
            intercept,
            slope,
        };
        servo.set_angle(0)?;
        Ok(servo)
    }

    pub fn set_angle(&self, angle: i64) -> Result<(), String> {
        if angle.abs() > (self.max_angle as i64) {
            Err(format!(
                "Provided angle '{}' is outside of valid range [{}, {}]",
                angle,
                -(self.max_angle as i64),
                self.max_angle
            ))
        } else {
            let pulse = self.slope * (angle as f64) + self.intercept;
            self.pwm
                .set_pulse_width(Duration::from_micros(pulse as u64))
                .map_err(|x| format!("Failed setting pulse width: {x}"))?;
            Ok(())
        }
    }
}

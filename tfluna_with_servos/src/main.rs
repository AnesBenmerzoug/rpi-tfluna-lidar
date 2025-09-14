use std::env;
use std::error::Error;
use std::thread;
use std::time::Duration;

use embedded_tfluna::{
    i2c::{I2CAddress, TFLuna},
    TFLunaSync,
};
use rerun::{self, external::arrow::compute::max};
use rppal::hal::Delay;
use rppal::i2c::I2c;
use rppal::pwm::{Channel, Polarity, Pwm};

// Servo configuration.
// minimum and maximum values.
//
/// Period: 20 ms (50 Hz)
const PERIOD_MS: u64 = 20;
// Position Left
const PULSE_MIN_US: u64 = 1000;
/// Position Center
const PULSE_NEUTRAL_BOTTOM_US: u64 = 1500;
const PULSE_NEUTRAL_TOP_US: u64 = 1525;
/// Position Right
const PULSE_MAX_US: u64 = 2000;
// Max Angle
const MAX_ANGLE_DEG: u64 = 45;

// Servo Channels
const BOTTOM_SERVO_CHANNEL: Channel = Channel::Pwm0;
const TOP_SERVO_CHANNEL: Channel = Channel::Pwm1;

struct ServoMotor {
    pwm: Pwm,
    max_angle: u64,
    intercept: f64,
    slope: f64,
}

impl ServoMotor {
    fn new(
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

    fn set_angle(&self, angle: i64) -> Result<(), String> {
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

fn main() -> Result<(), Box<dyn Error>> {
    // Connect to rerun server
    let rerun_server_ip = env::var("RERUN_SERVER_IP").unwrap_or(String::from("192.168.178.21"));
    let rec = rerun::RecordingStreamBuilder::new("rpi-lidar").connect_grpc_opts(
        format!("rerun+http://{}:9876/proxy", rerun_server_ip),
        rerun::default_flush_timeout(),
    )?;

    // Instantiate I2C peripheral
    let i2c = match I2c::new() {
        Ok(i2c) => i2c,
        Err(err) => {
            println!("Failed getting acces to I2c due to {}", err);
            panic!();
        }
    };

    let mut tfluna = TFLuna::new(i2c, I2CAddress::default(), Delay::new()).unwrap();
    tfluna.enable().unwrap();
    thread::sleep(Duration::from_millis(100));

    // Enable PWM channel 0 (BCM GPIO 12, physical pin 32) with the specified period,
    // and rotate the servo to the neutral position.
    let pwm_bottom = Pwm::with_period(
        BOTTOM_SERVO_CHANNEL,
        Duration::from_millis(PERIOD_MS),
        Duration::from_micros(PULSE_MAX_US),
        Polarity::Normal,
        true,
    )?;
    let servo_bottom = ServoMotor::new(
        pwm_bottom,
        PULSE_NEUTRAL_BOTTOM_US,
        PULSE_MAX_US,
        MAX_ANGLE_DEG,
    )?;

    // Enable PWM channel 1 (BCM GPIO 13, physical pin 33) with the specified period,
    // and rotate the servo to the neutral position.
    let pwm_top = Pwm::with_period(
        TOP_SERVO_CHANNEL,
        Duration::from_millis(PERIOD_MS),
        Duration::from_micros(PULSE_MAX_US),
        Polarity::Normal,
        true,
    )?;
    let servo_top = ServoMotor::new(
        pwm_top,
        PULSE_NEUTRAL_TOP_US,
        PULSE_MAX_US,
        MAX_ANGLE_DEG,
    )?;
    thread::sleep(Duration::from_millis(1000));

    for angle_bottom in (-(MAX_ANGLE_DEG as i64)..=(MAX_ANGLE_DEG as i64)).step_by(5) {
        println!("Bottom servo angle: {angle_bottom}");
        servo_bottom.set_angle(angle_bottom)?;
        thread::sleep(Duration::from_millis(200));
        for angle_top in (-(MAX_ANGLE_DEG as i64)..=(MAX_ANGLE_DEG as i64)).step_by(5) {
            println!("Top servo angle: {angle_top}");
            servo_top.set_angle(angle_top)?;
            thread::sleep(Duration::from_millis(200));
            
            let measurement = tfluna.measure().unwrap();
            println!("measurement = {:?}", measurement);
            rec.set_time_sequence("timestamp", measurement.timestamp);
            rec.log(
                "lidar/distance",
                &rerun::Scalars::single(measurement.distance),
            )?;
            rec.log(
                "lidar/signal_strength",
                &rerun::Scalars::single(measurement.signal_strength),
            )?;
            rec.log(
                "lidar/temperature",
                &rerun::Scalars::single(measurement.temperature),
            )?;
        }
    }
    // Go back to neutral positions
    servo_bottom.set_angle(0)?;
    servo_top.set_angle(0)?;
    thread::sleep(Duration::from_millis(2000));
    Ok(())
}

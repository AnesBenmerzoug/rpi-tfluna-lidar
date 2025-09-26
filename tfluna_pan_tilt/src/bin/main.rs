extern crate tfluna_pan_tilt;

use std::env;
use std::error::Error;
use std::thread;
use std::time::Duration;

use colorgrad::Gradient;
use embedded_tfluna::i2c::{Address, TFLuna};
use rerun;
use rppal::hal::Delay;
use rppal::i2c::I2c;
use rppal::pwm::{Channel, Polarity, Pwm};

use tfluna_pan_tilt::servo::ServoMotor;

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

    let mut tfluna = TFLuna::new(i2c, Address::default(), Delay::new()).unwrap();
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
    let servo_top = ServoMotor::new(pwm_top, PULSE_NEUTRAL_TOP_US, PULSE_MAX_US, MAX_ANGLE_DEG)?;
    thread::sleep(Duration::from_millis(1000));

    // Color gradient generator for point cloud
    let g = colorgrad::preset::spectral();
    // Vector to store all points
    let mut positions = Vec::new();
    let mut colors = Vec::new();

    let angle_step = 1;

    for angle_bottom in (-(MAX_ANGLE_DEG as i64)..=(MAX_ANGLE_DEG as i64)).step_by(angle_step) {
        println!("==========");
        println!("Bottom servo angle: {angle_bottom}");
        servo_bottom.set_angle(angle_bottom)?;
        thread::sleep(Duration::from_millis(200));

        for angle_top in (-(MAX_ANGLE_DEG as i64)..=(MAX_ANGLE_DEG as i64)).step_by(angle_step) {
            println!("----------");
            println!("Top servo angle: {angle_top}");
            servo_top.set_angle(angle_top)?;
            thread::sleep(Duration::from_millis(200));

            let measurement = tfluna.measure().unwrap();
            // Helper variables
            let yaw = (angle_bottom as f32).to_radians();
            let pitch = (angle_top as f32).to_radians();
            // Point 3D position
            let px = (measurement.distance as f32) * (pitch as f32).cos() * (yaw as f32).cos();
            let py = (measurement.distance as f32) * (pitch as f32).cos() * (yaw as f32).sin();
            let pz = (measurement.distance as f32) * (pitch as f32).sin();
            let position = [px, py, pz];
            // Point's color based on distance
            let color = g.at((measurement.distance as f32) / 800.0).to_rgba8();

            println!("distance = {}", measurement.distance);
            println!("position = {position:?}");
            positions.push(position);
            colors.push(color);

            rec.set_time("capture_time", std::time::SystemTime::now());
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
            rec.log(
                "lidar/position",
                &rerun::Points3D::new(positions.clone())
                    .with_colors(colors.clone())
                    .with_radii([3.0]),
            )?;
        }
    }
    // Go back to neutral positions
    servo_bottom.set_angle(0)?;
    servo_top.set_angle(0)?;
    thread::sleep(Duration::from_millis(1000));
    Ok(())
}

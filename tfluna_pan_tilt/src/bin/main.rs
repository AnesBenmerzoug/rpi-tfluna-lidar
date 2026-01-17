extern crate tfluna_pan_tilt;

use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use clap::Parser;
use colorgrad::Gradient;
use embedded_hal_bus::i2c::MutexDevice;
use embedded_tfluna::{
    RangingMode,
    i2c::{Address, TFLuna},
};
use pwm_pca9685::{Address as PWMAddress, Channel, Pca9685};
use rerun;
use rppal::hal::Delay;
use rppal::i2c::I2c;

use tfluna_pan_tilt::servo::ServoMotor;

// Servo Channels
const BOTTOM_SERVO_CHANNEL: Channel = Channel::C14;
const TOP_SERVO_CHANNEL: Channel = Channel::C15;

#[derive(clap::Parser, Debug)]
#[command(version = None, about = "Configurable TFLuna on Pan Tilt", long_about = None)]
struct Cli {
    #[arg(long, default_value_t = String::from("10.181.190.150"), help = "IP Address of a running rerun server")]
    rerun_server_ip: String,
    #[arg(
        long,
        default_value_t = 100,
        help = "Delay in milliseconds after servo motor command"
    )]
    servo_motor_delay: u32,
    #[arg(
        long,
        default_value_t = 30.0,
        help = "Size of servo motor angle increment in degrees"
    )]
    angle_step: f32,
    #[arg(
        long,
        default_value_t = -30.0,
        help = "Minimum angle for bottom servo motor"
    )]
    min_angle_bottom: f32,
    #[arg(
        long,
        default_value_t = 30.0,
        help = "Maximum angle for bottom servo motor"
    )]
    max_angle_bottom: f32,
    #[arg(
        long,
        default_value_t = -30.0,
        help = "Minimum angle for top servo motor"
    )]
    min_angle_top: f32,
    #[arg(
        long,
        default_value_t = 30.0,
        help = "Maximum angle for top servo motor"
    )]
    max_angle_top: f32,
    #[arg(
        long,
        default_value_t = 0.1,
        help = "Radius of points in centimeters for viewer"
    )]
    point_radius: f32,
    #[arg(
        long,
        default_value_t = 200.0,
        help = "Maximum distance in centimeters"
    )]
    maximum_distance: f32,
    #[arg(long, default_value_t = 10.0, help = "Minimum distance in centimeters")]
    minimum_distance: f32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();
    let rerun_server_ip = args.rerun_server_ip;
    let servo_motor_delay = Duration::from_millis(args.servo_motor_delay as u64);
    let angle_step = args.angle_step;
    // Instantiate I2C peripheral
    let i2c = match I2c::new() {
        Ok(i2c) => Mutex::new(i2c),
        Err(err) => {
            println!("Failed getting access to I2c due to {}", err);
            panic!();
        }
    };
    let i2c_tfluna = MutexDevice::new(&i2c);
    let mut tfluna = TFLuna::new(i2c_tfluna, Address::default(), Delay::new()).unwrap();
    tfluna.enable().unwrap();
    tfluna.set_ranging_mode(RangingMode::Trigger).unwrap();
    thread::sleep(Duration::from_millis(100));

    let i2c_servo = MutexDevice::new(&i2c);
    let address = PWMAddress::default();
    let mut pwm = Pca9685::new(i2c_servo, address).unwrap();
    // This corresponds to a frequency of 50 Hz.
    pwm.set_prescale(122).unwrap();
    // It is necessary to enable the device.
    pwm.enable().unwrap();

    let pwm = Rc::new(RefCell::new(pwm));

    let mut servo_bottom = ServoMotor::new(
        pwm.clone(),
        BOTTOM_SERVO_CHANNEL,
        args.min_angle_bottom,
        args.max_angle_bottom,
        true,
    )
    .unwrap();

    let mut servo_top = ServoMotor::new(
        pwm.clone(),
        TOP_SERVO_CHANNEL,
        args.min_angle_top,
        args.max_angle_top,
        true,
    )
    .unwrap();

    thread::sleep(Duration::from_millis(1000));

    // Color gradient generator for point cloud
    let g = colorgrad::preset::spectral();
    // Vector to store all points
    let mut positions = Vec::new();
    let mut colors = Vec::new();

    let mut angle_bottom = servo_bottom.get_min_angle();
    servo_bottom.set_angle(angle_bottom).unwrap();
    thread::sleep(Duration::from_millis(1000));

    // Rerun parameters
    let application_id = "rpi-lidar";
    let yaw_entity_path = "yaw";
    let pitch_entity_path = "pitch";
    let distance_entity_path = "distance";
    let signal_strength_entity_path = "signal_strength";
    let temperature_entity_path = "temperature";
    let position_entity_path = "position";

    // Connect to rerun server
    let rec = rerun::RecordingStreamBuilder::new(application_id).connect_grpc_opts(
        format!("rerun+http://{}:9876/proxy", rerun_server_ip),
        rerun::default_flush_timeout(),
    )?;
    rec.send_recording_name(format!(
        "{}deg-{}ms",
        args.angle_step, args.servo_motor_delay,
    ))?;
    rec.send_property(
        "servo_motor_delay",
        &rerun::Scalars::single(args.servo_motor_delay),
    )?;
    rec.send_property("angle_step", &rerun::Scalars::single(args.angle_step))?;

    // Variable used to determine whether the top servo
    // should go from top to bottom or from bottom to top
    let mut go_up = false;

    while servo_bottom.is_angle_allowed(angle_bottom) {
        servo_bottom.set_angle(angle_bottom).unwrap();
        thread::sleep(servo_motor_delay);

        let mut angle_top = if go_up {
            servo_top.get_min_angle()
        } else {
            servo_top.get_max_angle()
        };

        while servo_top.is_angle_allowed(angle_top) {
            servo_top.set_angle(angle_top).unwrap();
            thread::sleep(servo_motor_delay);

            tfluna.trigger_measurement().unwrap();
            thread::sleep(Duration::from_millis(20));
            let measurement = tfluna.get_measurement().unwrap();
            thread::sleep(Duration::from_millis(20));
            //println!("Yaw = {}, Pitch = {}, Distance = {}", angle_bottom, angle_top, measurement.distance);
            // Helper variables
            let yaw = (angle_bottom as f32).to_radians();
            let pitch = (angle_top as f32).to_radians();
            // Point 3D position
            let px = (measurement.distance as f32) * (pitch as f32).cos() * (yaw as f32).sin();
            let py = (measurement.distance as f32) * (pitch as f32).cos() * (yaw as f32).cos();
            let pz = (measurement.distance as f32) * (pitch as f32).sin();
            let position = [px, py, pz];
            // Point's color based on distance
            let color = g
                .at((measurement.distance as f32 - args.minimum_distance)
                    / (args.maximum_distance - args.minimum_distance))
                .to_rgba8();
            positions.push(position);
            colors.push(color);

            rec.set_time("capture_time", std::time::SystemTime::now());
            rec.log(yaw_entity_path, &rerun::Scalars::single(angle_bottom))?;
            rec.log(pitch_entity_path, &rerun::Scalars::single(angle_top))?;
            rec.log(
                distance_entity_path,
                &rerun::Scalars::single(measurement.distance),
            )?;
            rec.log(
                signal_strength_entity_path,
                &rerun::Scalars::single(measurement.signal_strength),
            )?;
            rec.log(
                temperature_entity_path,
                &rerun::Scalars::single(measurement.temperature),
            )?;
            rec.log(
                position_entity_path,
                &rerun::Points3D::new(positions.clone())
                    .with_colors(colors.clone())
                    .with_radii([args.point_radius]),
            )?;

            // Increment/decrement after recording the measurements
            if go_up {
                angle_top += angle_step;
            } else {
                angle_top -= angle_step;
            }
        }
        angle_bottom += angle_step;
        go_up = !go_up;
    }
    // Go back to neutral position
    servo_bottom.set_angle(0.0).unwrap();
    servo_top.set_angle(0.0).unwrap();
    thread::sleep(Duration::from_millis(1000));
    Ok(())
}

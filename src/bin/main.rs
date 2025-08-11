extern crate rpi_lidar;

use std::env;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

use rerun;
use rppal::i2c::I2c;

use rpi_lidar::tf_luna::TFLuna;

const TF_LUNA_ADDRESS: u16 = 0x10;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello Raspberry Pi!");

    let mut i2c = match I2c::new() {
        Ok(i2c) => i2c,
        Err(err) => {
            println!("Failed getting acces to I2c due to {}", err);
            panic!();
        }
    };
    match i2c.set_slave_address(TF_LUNA_ADDRESS) {
        Ok(_) => println!("Successfully set I2C slave address to {}", TF_LUNA_ADDRESS),
        Err(err) => {
            println!("Failed setting I2C slave address due to {}", err);
            panic!();
        }
    }
    let mut tf_luna = TFLuna::new(i2c)?;

    tf_luna.enable()?;

    tf_luna.show_raw_register_contents()?;

    let device_information = tf_luna.get_device_information()?;
    println!("Device information: {:?}", device_information);

    let rerun_server_ip = env::var("RERUN_SERVER_IP").unwrap_or(String::from("127.0.0.1"));

    let rec = rerun::RecordingStreamBuilder::new("rpi-lidar").connect_grpc_opts(
        format!("rerun+http://{}:9876/proxy", rerun_server_ip),
        rerun::default_flush_timeout(),
    )?;

    loop {
        let reading = tf_luna.read()?;
        println!("reading = {:?}", reading);
        rec.set_time_sequence("timestamp", reading.timestamp);
        rec.log(
            "rpi/lidar/distance",
            &rerun::Scalars::single(reading.distance),
        )?;
        rec.log(
            "rpi/lidar/signal_strength",
            &rerun::Scalars::single(reading.signal_strength),
        )?;
        rec.log(
            "rpi/lidar/temperature",
            &rerun::Scalars::single(reading.temperature),
        )?;
        sleep(Duration::from_millis(100));
    }
    Ok(())
}

use std::error::Error;
use std::time::Duration;
use std::thread;

use rppal::pwm::{Channel, Polarity, Pwm};

// SG90 Servo configuration.
// http://www.ee.ic.ac.uk/pcheung/teaching/DE1_EE/stores/sg90_datasheet.pdf
// minimum and maximum values.
//
/// Period: 20 ms (50 Hz)
const PERIOD_MS: u64 = 20;
// Position Left: Pulse width: 1000 µs
const PULSE_MIN_US: u64 = 1200;
/// Position 0: Pulse width: 1500 µs
const PULSE_NEUTRAL_US: u64 = 1500;
/// Position Right: 2000 µs.
const PULSE_MAX_US: u64 = 2000;

fn main() -> Result<(), Box<dyn Error>> {
    // Enable PWM channel 0 (BCM GPIO 12, physical pin 32) with the specified period,
    // and rotate the servo by setting the pulse width to its maximum value.
    let pwm = Pwm::with_period(
        Channel::Pwm0,
        Duration::from_millis(PERIOD_MS),
        Duration::from_micros(PULSE_MAX_US),
        Polarity::Normal,
        false,
    )?;

    // Sleep for 500 ms while the servo moves into position.
    println!("First wait");
    pwm.set_pulse_width(Duration::from_micros(PULSE_MIN_US))?;
    pwm.enable()?;
    thread::sleep(Duration::from_millis(2000));

    // Rotate the servo to the opposite side.
    pwm.set_pulse_width(Duration::from_micros(PULSE_MIN_US))?;
    println!("Min");
    thread::sleep(Duration::from_millis(2000));
    pwm.set_pulse_width(Duration::from_micros(1800))?;
    println!("Max");
    thread::sleep(Duration::from_millis(2000));
    pwm.set_pulse_width(Duration::from_micros(PULSE_NEUTRAL_US))?;
    println!("Neutral");
    thread::sleep(Duration::from_millis(2000));
    
    pwm.disable()?;
    println!("Finished");

    /*
    // Rotate the servo to its neutral (center) position in small steps.
    for pulse in (PULSE_MIN_US..=PULSE_NEUTRAL_US).step_by(10) {
        pwm.set_pulse_width(Duration::from_micros(pulse))?;
        thread::sleep(Duration::from_millis(500));
    } */

    Ok(())
}

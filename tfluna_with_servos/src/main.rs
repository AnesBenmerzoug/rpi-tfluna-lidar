use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::pwm::{Pwm, Polarity, Channel};

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

// Servo Channels
const BOTTOM_SERVO_CHANNEL: Channel = Channel::Pwm0;
const TOP_SERVO_CHANNEL: Channel = Channel::Pwm1;


fn main() -> Result<(), Box<dyn Error>> {
    // Enable PWM channel 0 (BCM GPIO 12, physical pin 32) with the specified period,
    // and rotate the servo to the neutral position.
    let pwm_bottom = Pwm::with_period(
        BOTTOM_SERVO_CHANNEL,
        Duration::from_millis(PERIOD_MS),
        Duration::from_micros(PULSE_MAX_US),
        Polarity::Normal,
        false,
    )?;
    pwm_bottom.set_pulse_width(Duration::from_micros(PULSE_NEUTRAL_BOTTOM_US))?;
    pwm_bottom.enable()?;

    // Enable PWM channel 1 (BCM GPIO 13, physical pin 33) with the specified period,
    // and rotate the servo to the neutral position.
    let pwm_top = Pwm::with_period(
        TOP_SERVO_CHANNEL,
        Duration::from_millis(PERIOD_MS),
        Duration::from_micros(PULSE_MAX_US),
        Polarity::Normal,
        false,
    )?;
    pwm_top.set_pulse_width(Duration::from_micros(PULSE_NEUTRAL_TOP_US))?;
    pwm_top.enable()?;
    thread::sleep(Duration::from_millis(1000));

    for pulse_bottom in (PULSE_MIN_US..=PULSE_MAX_US).step_by(100) {
        println!("Bottom servo pulse: {pulse_bottom}");
        pwm_bottom.set_pulse_width(Duration::from_micros(pulse_bottom))?;
        thread::sleep(Duration::from_millis(200));
        for pulse_top in (PULSE_MIN_US..=PULSE_MAX_US).step_by(100) {
            println!("Top servo pulse: {pulse_top}");
            pwm_top.set_pulse_width(Duration::from_micros(pulse_top))?;
            thread::sleep(Duration::from_millis(200));
        }
    }
    // Go back to neutral positions
    pwm_bottom.set_pulse_width(Duration::from_micros(PULSE_NEUTRAL_BOTTOM_US))?;
    pwm_top.set_pulse_width(Duration::from_micros(PULSE_NEUTRAL_TOP_US))?;
    thread::sleep(Duration::from_millis(2000));
    Ok(())
}

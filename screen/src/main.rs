use rppal::gpio::{Gpio, OutputPin, InputPin};
use rppal::spi::{Spi, Bus, SlaveSelect, Mode};
use std::thread::sleep;
use std::time::Duration;

const CS_PIN: u8 = 22;    // Chip Select
const BUSY_PIN: u8 = 24;  // Busy
const RESET_PIN: u8 = 26; // Reset


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize SPI
    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 4_000_000, Mode::Mode0)?;

    // Initialize GPIO pins
    let gpio = Gpio::new()?;
    let mut cs = gpio.get(CS_PIN)?.into_output();
    let busy = gpio.get(BUSY_PIN)?.into_input();
    let mut reset = gpio.get(RESET_PIN)?.into_output();

    // Reset the e-Paper display
    println!("Resetting e-Paper display...");
    reset.set_high();
    sleep(Duration::from_millis(200));
    reset.set_low();
    sleep(Duration::from_millis(200));
    reset.set_high();
    sleep(Duration::from_millis(200));

    // Wait for the display to be ready
    println!("Waiting for e-Paper display to be ready...");
    while busy.is_low() {
        sleep(Duration::from_millis(10));
        println!("Busy pin state: {}", busy.is_low());
    }

    // Send initialization command
    println!("Sending initialization command...");
    cs.set_low();
    let init_command = [0x00, 0x01]; // Replace with actual initialization command
    spi.write(&init_command)?;
    cs.set_high();

    // Wait for the display to process the command
    while busy.is_low() {
        sleep(Duration::from_millis(10));
    }

// Drawintln!("Test complete!");
    Ok(())
}

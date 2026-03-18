
use current_platform::{COMPILED_ON,CURRENT_PLATFORM};
use tokio::fs::File;
use tokio::time::{Instant, Duration,sleep};
use tokio::io::{AsyncReadExt,AsyncBufReadExt, BufReader};
use rppal::gpio::{Gpio, InputPin};
use rppal::uart::{Parity, Uart};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
// ########################################################
// DECLARATIONS
// ########################################################

const GPIO_FIX: u8 = 4; // Pin 4 has gps fix


enum GPSFix {
    NoFix,
    Fix,
}

impl GPSFix {
    fn is_fixed(&self) -> bool {
        match self {
            GPSFix::Fix => true,
            GPSFix::NoFix => false,
        }
    }
}


// #################################################################################################################
// FUNCTIONS
// #################################################################################################################

async fn check_fix_frequency(pin: &InputPin, last_pulse: &mut Option<Instant>, gpsfix: &mut GPSFix) {
    if pin.is_high() {
        if let Some(last_time) = *last_pulse {
            let elapsed = last_time.elapsed();
            println!("Time {:?}", elapsed);
            if elapsed.as_secs_f64() >= 29.0 && elapsed.as_secs_f64() <= 31.0 {
                *gpsfix = GPSFix::Fix;
                println!("GPS Fix acquired. Frequency: 1/15 Hz");
            } else if elapsed.as_secs_f64() >= 1.9 && elapsed.as_secs_f64() <= 2.1 {
                *gpsfix = GPSFix::NoFix;
                println!("No GPS Fix. Frequency: 1 Hz");
            }
        }
        *last_pulse = Some(Instant::now());
    }
    tokio::time::sleep(Duration::from_millis(100)).await; // Add a delay to avoid busy-waiting
}

async fn read_uart(uart: &mut Uart) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0u8; 1024];
    let mut uart_buffer = String::new();

    loop {
        match uart.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    uart_buffer.push_str(&String::from_utf8_lossy(&buffer[..bytes_read]));

                    while let Some(newline_idx) = uart_buffer.find('\n') {
                        let sentence = uart_buffer.drain(..=newline_idx).collect::<String>();

                        if sentence.starts_with('$') {
                            //println!("{}", sentence.trim());
                            nmea(sentence.trim());
                        } else {
                            eprintln!("Invalid sentence: {}", sentence.trim());
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error reading UART: {}", e),
        }

        sleep(Duration::from_millis(100)).await; // Add a delay to avoid busy-waiting
    }
}

fn nmea(sentence: &str) {
        // if !validate_checksum(sentence) {
        //     eprintln!("Invalid checksum for sentence: {}", sentence);
        //     return;
        // }
    
        if sentence.starts_with("$GPGGA") {
            let fields: Vec<&str> = sentence.split(',').collect();
            if fields.len() > 9 {
                let latitude = fields[2];
                let longitude = fields[4];
                let fix_quality = fields[6];
                println!("GPGGA -> Latitude: {}, Longitude: {}, Fix Quality: {}", latitude, longitude, fix_quality);
            }
        } else if sentence.starts_with("$GPRMC") {
            let fields: Vec<&str> = sentence.split(',').collect();
            if fields.len() > 9 {
                let latitude = fields[3];
                let longitude = fields[5];
                let speed_over_ground = fields[7];
                println!("GPRMC -> Latitude: {}, Longitude: {}, Speed: {}", latitude, longitude, speed_over_ground);
            }
        } else {
            // println!("Unknown NMEA sentence: {}", sentence);
            return;
        }
    }

//#####################################################################################################################
// MAIN
//#####################################################################################################################

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Print the current platform and compilation state for check
    println!("Bonjour Mondde!");
    println!(
        "I am async code beep boop. I am here, {}, and I was compiled on {}",
        CURRENT_PLATFORM, COMPILED_ON
    );

    let path: &str = "./pi_confirm.txt";

    // Open the file
    let mut file = File::open(path).await?;

    // Read the file contents into a String
    let mut content = String::new();
    file.read_to_string(&mut content).await?;

    let running = Arc::new(AtomicBool::new(true));
    let mut fix_pin = Gpio::new()?.get(GPIO_FIX)?.into_input_pullup();
    let mut gpsfix = GPSFix::NoFix;
    let mut fixtime: Option<Instant> = None;
    let mut uart = Uart::new(9600, Parity::None, 8, 1)?;
    uart.set_read_mode(1, Duration::default())?;

    let running_clone = running.clone();
    tokio::spawn(async move {
        while running_clone.load(Ordering::SeqCst) {
            check_fix_frequency(&fix_pin, &mut fixtime, &mut gpsfix).await;
            println!("GPS Fix status: {:?}", gpsfix.is_fixed());
        }
    });

    let running_clone = running.clone();
    tokio::spawn(async move {
        read_uart(&mut uart).await.unwrap();
    });

    // Wait for a signal to stop
    while running.load(Ordering::SeqCst) {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    println!("Exit triggered, closing code.");
    Ok(())
}
use rppal::gpio::{Gpio, OutputPin, InputPin};
use rppal::spi::{Spi, Bus, SlaveSelect, Mode};
use std::thread::sleep;
use std::time::Duration;

const CS_PIN: u8 = 22;    // Chip Select
const BUSY_PIN: u8 = 24;  // Busy
const RESET_PIN: u8 = 26; // Reset

// IT8951 SPI Commands
const IT8951_TCON_SYS_RUN: u8 = 0x01;       // System Run command
const IT8951_TCON_SET_VCOM: u8 = 0x39;      // Set VCOM command
const IT8951_TCON_REFRESH: u8 = 0x12;       // Refresh display command
const IT8951_TCON_LD_IMG: u8 = 0x10;        // Load image command
const IT8951_TCON_LD_IMG_AREA: u8 = 0x11;   // Load image to specific area command

const DISPLAY_REG_BASE: u32 = 0x1000; // Register RW access
const SYS_REG_BASE: u32 = 0x0000;     // System Registers Base Address
const MCSR_BASE_ADDR: u32 = 0x0200;   // Memory Converter Registers Base Address

const LUT0EWHR: u32 = DISPLAY_REG_BASE + 0x00;  // LUT0 Engine Width Height Reg
const LUT0XYR: u32 = DISPLAY_REG_BASE + 0x40;  // LUT0 XY Reg
const LUT0BADDR: u32 = DISPLAY_REG_BASE + 0x80; // LUT0 Base Address Reg
const LUT0MFN: u32 = DISPLAY_REG_BASE + 0xC0;  // LUT0 Mode and Frame number Reg
const UP0SR: u32 = DISPLAY_REG_BASE + 0x134;   // Update Parameter0 Setting Reg
const UPBBADDR: u32 = DISPLAY_REG_BASE + 0x17C; // Update Buffer Base Address
const LISAR: u32 = MCSR_BASE_ADDR + 0x0008;    // Load Image Start Address Reg

#[derive(Debug)]
struct IT8951DevInfo {
    width: u16,
    height: u16,
    img_buf_addr: u32, // Converted to u32 for compatibility with Rust
}

#[derive(Debug)]
struct IT8951Area {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

fn gpio_init() -> Result<(OutputPin, InputPin, OutputPin), Box<dyn std::error::Error>> {
    let gpio = Gpio::new()?;
    let cs = gpio.get(CS_PIN)?.into_output();
    let busy = gpio.get(BUSY_PIN)?.into_input();
    let reset = gpio.get(RESET_PIN)?.into_output();
    Ok((cs, busy, reset))
}

fn spi_init() -> Result<Spi, Box<dyn std::error::Error>> {
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 4_000_000, Mode::Mode0)?;
    Ok(spi)
}

fn spi_write_byte(spi: &mut Spi, value: u8) -> Result<(), Box<dyn std::error::Error>> {
    spi.write(&[value])?;
    Ok(())
}

fn DEV_SPI_Write_nByte(pData: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    spi.write(pData)?;
    Ok(())
}

fn delay_ms(ms: u64) {
    sleep(Duration::from_millis(ms));
}

fn reset_display(reset: &mut OutputPin) {
    reset.set_high();
    delay_ms(200);
    reset.set_low();
    delay_ms(200);
    reset.set_high();
    delay_ms(200);
}

fn is_busy(busy: &InputPin) -> bool {
    busy.is_low()
}

fn epd_init(spi: &mut Spi, cs: &mut OutputPin, busy: &InputPin, reset: &mut OutputPin) -> Result<(), Box<dyn std::error::Error>> {
    // Reset the display
    reset_display(reset);

    // Send System Run command
    cs.set_low();
    let system_run_command = [IT8951_TCON_SYS_RUN]; // Replace with actual System Run command
    spi.write(&system_run_command)?;
    cs.set_high();

    // Wait for the display to be ready
    while is_busy(busy) {
        delay_ms(10);
    }

    // Set VCOM
    cs.set_low();
    let set_vcom_command = [IT8951_TCON_SET_VCOM, 0x00, 0x00]; // Replace with actual Set VCOM command
    spi.write(&set_vcom_command)?;
    cs.set_high();

    // Wait for the display to be ready
    while is_busy(busy) {
        delay_ms(10);
    }

    Ok(())
}

fn epd_system_run(spi: &mut Spi, cs: &mut OutputPin, busy: &InputPin) -> Result<(), Box<dyn std::error::Error>> {
    // Wait for the display to be ready
    while is_busy(busy) {
        delay_ms(10);
    }

    // Send System Run command
    println!("Sending System Run command...");
    cs.set_low();
    let command = [IT8951_TCON_SYS_RUN];
    spi.write(&command)?;
    cs.set_high();

    // Wait for the display to process the command
    while is_busy(busy) {
        delay_ms(10);
    }

    println!("System Run command executed successfully.");
    Ok(())
}

const VCOM_VOLTAGE: u16 = 0x2900; // -2.9V represented in hexadecimal

fn epd_set_vcom(spi: &mut Spi, cs: &mut OutputPin, busy: &InputPin) -> Result<(), Box<dyn std::error::Error>> {
    while is_busy(busy) {
        delay_ms(10);
    }

    cs.set_low();
    let command = [
        IT8951_TCON_SET_VCOM,
        (VCOM_VOLTAGE >> 8) as u8, // High byte of VCOM
        (VCOM_VOLTAGE & 0xFF) as u8, // Low byte of VCOM
    ];
    spi.write(&command)?;
    cs.set_high();

    while is_busy(busy) {
        delay_ms(10);
    }

    println!("VCOM set to -{} mV successfully.", VCOM_VOLTAGE);
    Ok(())
}

fn epd_refresh(spi: &mut Spi, cs: &mut OutputPin, busy: &InputPin) -> Result<(), Box<dyn std::error::Error>> {
    // Wait for the display to be ready
    while is_busy(busy) {
        delay_ms(10);
    }

    // Send Refresh Display command
    println!("Refreshing display...");
    cs.set_low();
    let command = [IT8951_TCON_REFRESH];
    spi.write(&command)?;
    cs.set_high();

    // Wait for the display to process the command
    while is_busy(busy) {
        delay_ms(10);
    }

    println!("Display refreshed successfully.");
    Ok(())
}

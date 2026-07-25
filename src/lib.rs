//! This crate is an embedded-hal driver library implementation for the Texas Instruments 80501,
//! 70501 and 60501 DACs. It relies on the embedded-hal 1.0.0 traits being implemented in
//! the board hal. See the [product page](https://www.ti.com/product/DAC80501/part-details/DAC80501ZDQFT) for the datasheet and other notes.

#![no_std]
#![deny(missing_docs)]
#![doc(
    html_logo_url = "https://www.ti.com/content/dam/ticom/images/products/package/d/dqf0008a.png"
)]

use core::fmt;

use embedded_hal::spi::SpiDevice;

/// Command byte, first byte of the transfer to the DAC
///
/// B23 B22 B21 B20 B19 B18 B17 B16 REGISTER     HEX
///  0   0   0   0   0   0   0   0   NOOP        0x00
///  0   0   0   0   0   0   0   1   DEVID       0x01
///  0   0   0   0   0   0   1   0   SYNC        0x02
///  0   0   0   0   0   0   1   1   CONFIG      0x03
///  0   0   0   0   0   1   0   0   GAIN        0x04
///  0   0   0   0   0   1   0   1   TRIGGER     0x05
///  0   0   0   0   0   1   1   1   STATUS      0x07
///  0   0   0   0   1   0   0   0   DACDATA     0x08
#[allow(dead_code)]
#[repr(u8)]
enum Command {
    NOOP = 0b0000_0000,
    DEVID = 0b0000_0001,
    SYNC = 0b0000_0010,
    CONFIG = 0b0000_0011,
    GAIN = 0b0000_0100,
    TRIGGER = 0b0000_0101,
    STATUS = 0b0000_0111,
    DACDATA = 0b0000_1000,
}

/// DAC Configuration
// TODO: Document
#[derive(Default)]
struct DACConfig {
    dac_power: PowerState,
    ref_power: InternalReference,
    ref_divider: ReferenceDivider,
    buffer_gain: BufferGain,
    _dac_code: u16,
}

/// DAC power state. When powered down the DAC output is connected to ground through a 1k resistor.
/// The device default is [`PowerState::On`]
#[derive(Default, Clone, Copy)]
pub enum PowerState {
    /// Normal operation
    #[default]
    On,
    /// Power down, output connected to ground
    Down,
}

/// Output buffer gain.
/// Power on value is [`BufferGain::Two`]
#[derive(Default, Clone, Copy)]
pub enum BufferGain {
    /// The output voltage of the device is [0 .. VREF]
    None,
    /// The output voltage of the device is [0 .. 2*VREF]
    #[default]
    Two,
}

/// DAC reference divider which applies to both internal and external reference sources.
/// Power on value is [`ReferenceDivider::None`]
#[derive(Default, Clone, Copy)]
pub enum ReferenceDivider {
    /// The reference voltage is not modified
    #[default]
    None,
    /// The reference voltage is divided by 2
    Two,
}

/// Status of the internal reference.
/// Power on value is [`InternalReference::Enabled`]
#[derive(Default, Clone, Copy)]
pub enum InternalReference {
    /// The device internal reference is enabled
    #[default]
    Enabled,
    /// The device internal reference is disabled. External reference must be provided.
    Disabled,
}

/// Synchronous (triggered), or asynchronous (continuous) output of a value loaded into the DACDATA register.
/// Synchronous output is triggered by writing to the LDAC bit of the trigger register.
/// Power on value is [`Mode::Asynchronous`]
#[derive(Default, Clone, Copy)]
pub enum Mode {
    /// The device internal reference is enabled
    #[default]
    Asynchronous,
    /// The device internal reference is disabled. External reference must be provided.
    Synchronous,
}

#[derive(PartialEq, Eq, Clone, Copy)]
/// Alarm when supply voltage is below what is required to power the internal reference and gain buffer. DAC outputs 0 volts while supply is too low.
/// Upon supply exceeding the analog threshold DAC output returns to normal operation with the output code unaffected.
/// Power on value is [`AlarmStatus::Normal`]
pub enum AlarmStatus {
    /// Not enough headroom, reference buffer shutdown. DAC outputs 0 volts.
    Alarm,
    /// Normal operation
    Normal,
}

#[derive(Debug)]
/// The custom error for this crate
pub enum DacError<E> {
    /// The value for the specified DAC overflowed
    ValueOverflow,
    /// An internal embedded hal SPI transfer error
    SpiError(E),
}
impl<E: fmt::Debug> fmt::Display for DacError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValueOverflow => write!(f, "The data value was too large for the selected DAC"),
            Self::SpiError(e) => write!(f, "SPI error: {:?}", e),
        }
    }
}

/// DAC TYPES:
pub struct DAC<SPI, const BITS: u8> {
    spi: SPI,
    config: DACConfig,
}

/// Dac80501, 16 bit DAC
pub type Dac80501<SPI> = DAC<SPI, 16>;
/// Dac70501, 14 bit DAC
pub type Dac70501<SPI> = DAC<SPI, 14>;
/// Dac60501, 12 bit DAC
pub type Dac60501<SPI> = DAC<SPI, 12>;

impl<SPI, const BITS: u8> DAC<SPI, BITS>
where
    SPI: SpiDevice,
{
    /// Width of the DACDATA shift register in bits. Fixed by the hardware
    /// regardless of the DAC resolution. 12 and 14 bit DACs left justify output code within output register.
    const REGISTER_WIDTH: u8 = 16;

    /// Creates a new instance of the specified dac with the internal state set to match
    /// the device defaults
    pub fn new(spi: SPI) -> Self {
        Self {
            spi,
            config: DACConfig::default(),
        }
    }

    /// Write to the NOOP register, has no effects
    pub fn set_noop(&mut self) -> Result<(), DacError<SPI::Error>> {
        self.spi
            .write(&[Command::NOOP as u8, 0x00, 0x00])
            .map_err(DacError::SpiError)?;
        Ok(())
    }

    /// Set whether the DAC is triggered by load DAC or if it is set to update immediately
    pub fn set_synchronous(&mut self, mode: Mode) -> Result<(), DacError<SPI::Error>> {
        self.spi
            .write(&[Command::SYNC as u8, 0x00, mode as u8])
            .map_err(DacError::SpiError)?;
        Ok(())
    }

    /// Enables and disables the device internal reference. The internal reference is on by default
    pub fn set_internal_reference(
        &mut self,
        intern_ref: InternalReference,
    ) -> Result<(), DacError<SPI::Error>> {
        self.config.ref_power = intern_ref;
        self.spi
            .write(&[
                Command::CONFIG as u8,
                self.config.ref_power as u8,
                self.config.dac_power as u8,
            ])
            .map_err(DacError::SpiError)?;
        Ok(())
    }

    /// In power-off state the device output is connected to GND through a 1-kΩ internal
    /// resistor. The device is in power `On` state by default. This reduces current
    /// consumption to typically 15 µA at 5 V.
    pub fn set_power_state(&mut self, state: PowerState) -> Result<(), DacError<SPI::Error>> {
        self.config.dac_power = state;
        self.spi
            .write(&[
                Command::CONFIG as u8,
                self.config.ref_power as u8,
                self.config.dac_power as u8,
            ])
            .map_err(DacError::SpiError)?;
        Ok(())
    }

    /// The reference voltage to the device (either from the internal or external reference) can be
    /// divided by a factor of two by setting the reference divider to `Half`. Make sure to configure
    /// the reference divider so that there is sufficient headroom from VDD to the DAC operating
    /// reference voltage. Improper configuration of the reference divider triggers a reference
    /// alarm condition. In the case of an alarm condition, the reference buffer is shut down, and
    /// all the DAC outputs go to 0 V. The DAC data registers are unaffected by the alarm
    /// condition, and thus enable the DAC output to return to normal operation after the reference
    /// divider is configured correctly. When the reference divider is set to `Half`, the reference
    /// voltage is internally divided by a factor of 2. The reference divider is set to `OneX` by
    /// default
    pub fn set_reference_divider(
        &mut self,
        ref_div: ReferenceDivider,
    ) -> Result<(), DacError<SPI::Error>> {
        self.config.ref_divider = ref_div;
        self.spi
            .write(&[
                Command::GAIN as u8,
                self.config.ref_divider as u8,
                self.config.buffer_gain as u8,
            ])
            .map_err(DacError::SpiError)?;
        Ok(())
    }

    /// When set to `TwoX`, the buffer amplifier for the DAC has a gain of 2x doubling the
    /// voltage output. When set to `OneX` it has a gain of 1x. Using this gain can be
    /// especially useful when using the internal reference divider set to `Half`. The
    /// output gain is set to `TwoX` by default
    pub fn set_output_gain(&mut self, gain: BufferGain) -> Result<(), DacError<SPI::Error>> {
        self.config.buffer_gain = gain;
        self.spi
            .write(&[
                Command::GAIN as u8,
                self.config.ref_divider as u8,
                self.config.buffer_gain as u8,
            ])
            .map_err(DacError::SpiError)?;
        Ok(())
    }

    /// Trigger synchronous load. Self resetting after load is completed. No effect for asynchronous operation.
    pub fn set_load_dac(&mut self) -> Result<(), DacError<SPI::Error>> {
        self.spi
            .write(&[Command::TRIGGER as u8, 0x00, 0b000_1_0000])
            .map_err(DacError::SpiError)?;
        Ok(())
    }

    /// Soft reset, reset device to power on defaults.
    pub fn soft_reset(&mut self) -> Result<(), DacError<SPI::Error>> {
        self.spi
            .write(&[Command::TRIGGER as u8, 0x00, 0b0000_1010])
            .map_err(DacError::SpiError)?;
        self.config = DACConfig::default();
        Ok(())
    }

    /// Set the output voltage of the device and check the level bounds for the specified device
    pub fn set_output_level(&mut self, level: u16) -> Result<(), DacError<SPI::Error>> {
        // Shifts to ensure level is not out of range for the number of bits the DAC has.
        // Check should be optimized out in the case of a 16bit DAC
        if level.checked_shr(BITS as u32).unwrap_or(0) != 0 {
            return Err(DacError::ValueOverflow);
        }

        let bytes = (level << (Self::REGISTER_WIDTH - BITS)).to_be_bytes();
        self.spi
            .write(&[Command::DACDATA as u8, bytes[0], bytes[1]])
            .map_err(DacError::SpiError)?;
        Ok(())
    }

    /// This function sets the output level without checking the bounds on the size of the
    /// value for the specified DAC
    pub fn set_output_level_unchecked(&mut self, level: u16) -> Result<(), DacError<SPI::Error>> {
        let bytes = (level << (Self::REGISTER_WIDTH - BITS)).to_be_bytes();
        self.spi
            .write(&[Command::DACDATA as u8, bytes[0], bytes[1]])
            .map_err(DacError::SpiError)?;
        Ok(())
    }
}

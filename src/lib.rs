//! This crate is an embedded-hal driver library implementation for the Texas Instruments 80501,
//! 70501 and 60501 DACs. It relies on the embedded-hal 1.0.0 traits being implemented in
//! the board hal. See the [product page](https://www.ti.com/product/DAC80501/part-details/DAC80501ZDQFT) for the datasheet and other notes.

#![no_std]
#![deny(missing_docs)]
#![doc(
    html_logo_url = "https://www.ti.com/content/dam/ticom/images/products/package/d/dqf0008a.png"
)]

use core::fmt;

use embedded_hal::i2c::I2c;
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
#[repr(u8)]
pub enum Register {
    /// NOOP Register
    NOOP = 0b0000_0000,
    /// DEVID Register
    DEVID = 0b0000_0001,
    /// SYNC Register
    SYNC = 0b0000_0010,
    /// CONFIG Register
    CONFIG = 0b0000_0011,
    /// GAIN Register
    GAIN = 0b0000_0100,
    /// TRIGGER Register
    TRIGGER = 0b0000_0101,
    /// STATUS Register
    STATUS = 0b0000_0111,
    /// DAC Register
    DACDATA = 0b0000_1000,
}

/// DAC Configuration
#[derive(Default)]
struct DACConfig {
    dac_power: PowerState,
    ref_power: InternalReference,
    ref_divider: ReferenceDivider,
    buffer_gain: BufferGain,
}

/// DAC power state. When powered down the DAC output is connected to ground through a 1k resistor.
/// The device default is [`PowerState::On`]
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PowerState {
    /// Normal operation
    #[default]
    On,
    /// Power down, output connected to ground
    Down,
}

/// Output buffer gain.
/// Power on value is [`BufferGain::Two`]
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BufferGain {
    /// The output voltage of the device is [0 .. VREF]
    None,
    /// The output voltage of the device is [0 .. 2*VREF]
    #[default]
    Two,
}

/// DAC reference divider which applies to both internal and external reference sources.
/// Power on value is [`ReferenceDivider::None`]
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReferenceDivider {
    /// The reference voltage is not modified
    #[default]
    None,
    /// The reference voltage is divided by 2
    Two,
}

/// Status of the internal reference.
/// Power on value is [`InternalReference::Enabled`]
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
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
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// The device internal reference is enabled
    #[default]
    Asynchronous,
    /// The device internal reference is disabled. External reference must be provided.
    Synchronous,
}

/// Reset value of the DAC output on power on reset.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResetValue {
    /// DAC output is 0 volts
    Zero,
    /// DAC output is mid scale
    MidScale,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
    /// An internal embedded hal I2C transfer error
    I2cError(E),
}
impl<E: fmt::Debug> fmt::Display for DacError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValueOverflow => write!(f, "The data value was too large for the selected DAC"),
            Self::SpiError(e) => write!(f, "SPI error: {:?}", e),
            Self::I2cError(e) => write!(f, "I2C error: {:?}", e),
        }
    }
}

/// Generic interface implementation. Allows the device to be setup using either SPI or I2C with the
/// same API.
pub trait Interface {
    /// Error type, Either SpiError or I2cError
    type Error;
    /// Write to a register, data is ordered high:low
    fn write_register(&mut self, c: Register, data: [u8; 2]) -> Result<(), DacError<Self::Error>>;
}

/// Wrapper over an SPI interface, holding the embedded-hal spi data
pub struct SpiInterface<SPI> {
    spi: SPI,
}
impl<SPI: SpiDevice> Interface for SpiInterface<SPI> {
    type Error = SPI::Error;

    fn write_register(&mut self, c: Register, data: [u8; 2]) -> Result<(), DacError<SPI::Error>> {
        self.spi
            .write(&[c as u8, data[0], data[1]])
            .map_err(DacError::SpiError)
    }
}

/// Wrapper over an I2C interface, holding the embedded-hal i2c data and the device address
pub struct I2cInterface<I2C> {
    i2c: I2C,
    address: u8,
}
impl<I2C: I2c> Interface for I2cInterface<I2C> {
    type Error = I2C::Error;

    fn write_register(&mut self, c: Register, data: [u8; 2]) -> Result<(), DacError<Self::Error>> {
        self.i2c
            .write(self.address, &[c as u8, data[0], data[1]])
            .map_err(DacError::I2cError)
    }
}

/// DAC TYPES:
pub struct DAC<I, const BITS: u8> {
    interface: I,
    config: DACConfig,
}

impl<SPI: SpiDevice, const BITS: u8> DAC<SpiInterface<SPI>, BITS> {
    /// Creates a new instance of the specified dac with the internal state set to match
    /// the device defaults using an SPI interface
    pub fn new_spi(spi: SPI) -> Self {
        Self {
            interface: SpiInterface { spi },
            config: DACConfig::default(),
        }
    }
}

/// Dac80501, 16 bit DAC,
pub type Dac80501<I> = DAC<I, 16>;
/// Dac70501, 14 bit DAC
pub type Dac70501<I> = DAC<I, 14>;
/// Dac60501, 12 bit DAC
pub type Dac60501<I> = DAC<I, 12>;

impl<I2C: I2c, const BITS: u8> DAC<I2cInterface<I2C>, BITS> {
    /// Creates a new instance of the specified dac with the internal state set to match
    /// the device defaults using an I2C interface
    pub fn new_i2c(i2c: I2C, address: u8) -> Self {
        Self {
            interface: I2cInterface { i2c, address },
            config: DACConfig::default(),
        }
    }

    fn read_register(&mut self, c: Register) -> Result<[u8; 2], DacError<I2C::Error>> {
        let mut buf = [0u8; 2];
        self.interface
            .i2c
            .write_read(self.interface.address, &[c as u8], &mut buf)
            .map_err(DacError::I2cError)?;
        Ok(buf)
    }

    /// Returns the resolution of the device in bits
    pub fn get_resolution(&mut self) -> Result<u8, DacError<I2C::Error>> {
        let buf = self.read_register(Register::DEVID)?;
        match buf[0] >> 4 {
            0b000 => Ok(16),
            0b001 => Ok(14),
            0b010 => Ok(12),
            _ => Err(DacError::ValueOverflow),
        }
    }

    /// Returns the power on reset value of the DAC
    pub fn get_reset_value(&mut self) -> Result<ResetValue, DacError<I2C::Error>> {
        let buf = self.read_register(Register::DEVID)?;
        match buf[1] >> 7 {
            0b00 => Ok(ResetValue::Zero),
            0b01 => Ok(ResetValue::MidScale),
            _ => Err(DacError::ValueOverflow),
        }
    }

    /// Returns whether the device is in synchronous or asynchronous mode
    pub fn get_synchronous(&mut self) -> Result<Mode, DacError<I2C::Error>> {
        let buf = self.read_register(Register::SYNC)?;
        match buf[1] & 0b0000000_1 {
            0b0 => Ok(Mode::Asynchronous),
            0b1 => Ok(Mode::Synchronous),
            _ => Err(DacError::ValueOverflow),
        }
    }

    /// Returns the whether the internal reference is enabled or disabled
    pub fn get_internal_reference(&mut self) -> Result<InternalReference, DacError<I2C::Error>> {
        let buf = self.read_register(Register::CONFIG)?;
        match buf[0] & 0b0000000_1 {
            0b0 => Ok(InternalReference::Enabled),
            0b1 => Ok(InternalReference::Disabled),
            _ => Err(DacError::ValueOverflow),
        }
    }

    /// Returns the DAC power state, on or down
    pub fn get_power_state(&mut self) -> Result<PowerState, DacError<I2C::Error>> {
        let buf = self.read_register(Register::CONFIG)?;
        match buf[1] & 0b0000000_1 {
            0b0 => Ok(PowerState::On),
            0b1 => Ok(PowerState::Down),
            _ => Err(DacError::ValueOverflow),
        }
    }

    /// Returns the status of the reference divider, either no reference division or divide by two
    pub fn get_reference_divider(&mut self) -> Result<ReferenceDivider, DacError<I2C::Error>> {
        let buf = self.read_register(Register::GAIN)?;
        match buf[0] & 0b0000000_1 {
            0b0 => Ok(ReferenceDivider::None),
            0b1 => Ok(ReferenceDivider::Two),
            _ => Err(DacError::ValueOverflow),
        }
    }

    /// Returns the output buffer gain either no gain or two times gain
    pub fn get_output_gain(&mut self) -> Result<BufferGain, DacError<I2C::Error>> {
        let buf = self.read_register(Register::GAIN)?;
        match buf[1] & 0b0000000_1 {
            0b0 => Ok(BufferGain::None),
            0b1 => Ok(BufferGain::Two),
            _ => Err(DacError::ValueOverflow),
        }
    }

    /// Returns reference alarm status. Alarm occurs when supply is below what is required to output the maximum output voltage.
    pub fn get_alarm_status(&mut self) -> Result<AlarmStatus, DacError<I2C::Error>> {
        let buf = self.read_register(Register::STATUS)?;
        match buf[1] & 0b0000000_1 {
            0b0 => Ok(AlarmStatus::Normal),
            0b1 => Ok(AlarmStatus::Alarm),
            _ => Err(DacError::ValueOverflow),
        }
    }

    /// Returns the current output level of the DAC
    pub fn get_output_level(&mut self) -> Result<u16, DacError<I2C::Error>> {
        let buf = self.read_register(Register::DACDATA)?;
        Ok(u16::from_be_bytes(buf) >> (Self::REGISTER_WIDTH - BITS))
    }
}

impl<I, const BITS: u8> DAC<I, BITS>
where
    I: Interface,
{
    /// Width of the DACDATA shift register in bits. Fixed by the hardware
    /// regardless of the DAC resolution. 12 and 14 bit DACs left justify output code within output register.
    const REGISTER_WIDTH: u8 = 16;

    /// Write to the NOOP register, has no effects
    pub fn set_noop(&mut self) -> Result<(), DacError<I::Error>> {
        self.interface.write_register(Register::NOOP, [0x00, 0x00])
    }

    /// Set whether the DAC is triggered by load DAC or if it is set to update immediately
    pub fn set_synchronous(&mut self, mode: Mode) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(Register::SYNC, [0x00, mode as u8])
    }

    /// Enables and disables the device internal reference. The internal reference is on by default
    pub fn set_internal_reference(
        &mut self,
        intern_ref: InternalReference,
    ) -> Result<(), DacError<I::Error>> {
        self.config.ref_power = intern_ref;
        self.interface.write_register(
            Register::CONFIG,
            [self.config.ref_power as u8, self.config.dac_power as u8],
        )
    }

    /// In power-off state the device output is connected to GND through a 1-kΩ internal
    /// resistor. The device is in power `On` state by default. This reduces current
    /// consumption to typically 15 µA at 5 V.
    pub fn set_power_state(&mut self, state: PowerState) -> Result<(), DacError<I::Error>> {
        self.config.dac_power = state;
        self.interface.write_register(
            Register::CONFIG,
            [self.config.ref_power as u8, self.config.dac_power as u8],
        )
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
    ) -> Result<(), DacError<I::Error>> {
        self.config.ref_divider = ref_div;
        self.interface.write_register(
            Register::GAIN,
            [self.config.ref_divider as u8, self.config.buffer_gain as u8],
        )
    }

    /// When set to `TwoX`, the buffer amplifier for the DAC has a gain of 2x doubling the
    /// voltage output. When set to `OneX` it has a gain of 1x. Using this gain can be
    /// especially useful when using the internal reference divider set to `Half`. The
    /// output gain is set to `TwoX` by default
    pub fn set_output_gain(&mut self, gain: BufferGain) -> Result<(), DacError<I::Error>> {
        self.config.buffer_gain = gain;
        self.interface.write_register(
            Register::GAIN,
            [self.config.ref_divider as u8, self.config.buffer_gain as u8],
        )
    }

    /// Trigger synchronous load. Self resetting after load is completed. No effect for asynchronous operation.
    pub fn set_load_dac(&mut self) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(Register::TRIGGER, [0x00, 0b000_1_0000])
    }

    /// Soft reset, reset device to power on defaults.
    pub fn soft_reset(&mut self) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(Register::TRIGGER, [0x00, 0b0000_1010])?;
        // Reset internal state ONLY if write was successful.
        self.config = DACConfig::default();
        Ok(())
    }

    /// Set the output voltage of the device and check the level bounds for the specified device
    pub fn set_output_level(&mut self, level: u16) -> Result<(), DacError<I::Error>> {
        // Shifts to ensure level is not out of range for the number of bits the DAC has.
        // Check should be optimized out in the case of a 16bit DAC
        if level.checked_shr(BITS as u32).unwrap_or(0) != 0 {
            return Err(DacError::ValueOverflow);
        }

        let bytes: [u8; 2] = (level << (Self::REGISTER_WIDTH - BITS)).to_be_bytes();
        self.interface.write_register(Register::DACDATA, bytes)
    }

    /// This function sets the output level without checking the bounds on the size of the
    /// value for the specified DAC
    pub fn set_output_level_unchecked(&mut self, level: u16) -> Result<(), DacError<I::Error>> {
        let bytes: [u8; 2] = (level << (Self::REGISTER_WIDTH - BITS)).to_be_bytes();
        self.interface.write_register(Register::DACDATA, bytes)
    }
}

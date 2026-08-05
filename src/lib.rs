#![doc = include_str!("../README.md")]

#![no_std]
#![deny(missing_docs)]
#![doc(
    html_logo_url = "https://www.ti.com/content/dam/ticom/images/products/package/d/dqf0008a.png"
)]

use core::fmt;

#[cfg(feature = "sync")]
use embedded_hal::i2c::I2c as I2cSync;
#[cfg(feature = "sync")]
use embedded_hal::spi::SpiDevice as SpiDeviceSync;

#[cfg(feature = "async")]
use embedded_hal_async::i2c::I2c as I2cAsync;
#[cfg(feature = "async")]
use embedded_hal_async::spi::SpiDevice as SpiDeviceAsync;

/// Register byte. first byte of the transfer to the DAC
// B23 B22 B21 B20 B19 B18 B17 B16 REGISTER     HEX
//  0   0   0   0   0   0   0   0   NOOP        0x00
//  0   0   0   0   0   0   0   1   DEVID       0x01
//  0   0   0   0   0   0   1   0   SYNC        0x02
//  0   0   0   0   0   0   1   1   CONFIG      0x03
//  0   0   0   0   0   1   0   0   GAIN        0x04
//  0   0   0   0   0   1   0   1   TRIGGER     0x05
//  0   0   0   0   0   1   1   1   STATUS      0x07
//  0   0   0   0   1   0   0   0   DACDATA     0x08
// Datasheet page 27
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
struct DacConfig {
    dac_power: PowerState,
    ref_power: InternalReference,
    ref_divider: ReferenceDivider,
    buffer_gain: BufferGain,
}

/// Power state of the device
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum PowerState {
    /// Power on reset value: Normal operation
    #[default]
    On = 0,
    /// Power down, output connected to ground through 1kOhm resistor
    Down = 1,
}

/// Gain of the output buffer amplifier
///
/// [`BufferGain`], [`ReferenceDivider`], and the reference voltage control
/// the full scale output range of the DAC. The full scale range is:
/// `VOUT = VREF * BufferGain / ReferenceDivider`
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum BufferGain {
    /// The output voltage is buffered but not amplified
    None = 0,
    /// Power on reset value: Output is doubled and buffered
    #[default]
    Two = 1,
}
/// Controls the reference voltage division
///
/// [`BufferGain`], [`ReferenceDivider`], and the reference voltage control
/// the full scale output range of the DAC. The full scale range is:
/// `VOUT = VREF * BufferGain / ReferenceDivider`
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ReferenceDivider {
    /// Power on reset value: The reference voltage is not modified
    #[default]
    None = 0,
    /// The reference voltage is divided by 2
    Two = 1,
}

/// Power state of the internal reference
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum InternalReference {
    /// Power on reset value: The device internal reference is enabled
    #[default]
    Enabled = 0,
    /// The device internal reference is disabled. External reference must be provided.
    Disabled = 1,
}

/// Controls whether the DAC continuously updates, or has triggered updates
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum UpdateMode {
    /// Power on reset value: The DAC output updates as soon as a write to the DACDATA register completes
    #[default]
    Asynchronous = 0,
    /// The DAC output does not update from the DACDATA register until a [`Dac80501::load_dac()`] command is issued
    Synchronous = 1,
}

/// The power on reset output value of the DAC
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ResetValue {
    /// DAC output is 0 volts, DACx0501Z variants
    Zero = 0,
    /// DAC output is mid scale, DACx0501M variants
    MidScale = 1,
}

/// Reference alarm state
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
pub enum AlarmStatus {
    /// Normal operation
    Normal = 0,
    /// Not enough supply headroom, reference buffer shutdown. DAC outputs 0 volts.
    Alarm = 1,
}

#[derive(Debug)]
/// The custom error for this crate
pub enum DacError<E> {
    /// The value for the specified DAC overflowed
    ValueOverflow,
    /// Unknown value for register
    UnknownValue,
    /// An error on the SPI or I2C interface
    InterfaceError(E),
}
impl<E: fmt::Debug> fmt::Display for DacError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValueOverflow => write!(f, "The data value was too large for the selected DAC"),
            Self::UnknownValue => write!(f, "Unknown value for register"),
            Self::InterfaceError(e) => write!(f, "Interface error: {:?}", e),
        }
    }
}

impl<E: fmt::Debug + core::error::Error + 'static> core::error::Error for DacError<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::ValueOverflow => None,
            Self::UnknownValue => None,
            Self::InterfaceError(e) => Some(e),
        }
    }
}

/// Generic interface implementation. Allows the device to be setup using either SPI or I2C with the
/// same API.
///
#[allow(async_fn_in_trait)]
#[maybe_async_cfg::maybe(idents(Interface), sync(feature = "sync"), async(feature = "async"))]
pub trait Interface {
    /// Error type, Either SpiError or I2cError
    type Error;
    /// Write to a register, data is ordered high:low
    async fn write_register(
        &mut self,
        c: Register,
        data: [u8; 2],
    ) -> Result<(), DacError<Self::Error>>;
}

/// Wrapper over an SPI interface, holding the embedded-hal spi data
#[maybe_async_cfg::maybe(idents(SpiInterface), sync(feature = "sync"), async(feature = "async"))]
pub struct SpiInterface<SPI> {
    spi: SPI,
}
#[maybe_async_cfg::maybe(
    idents(SpiDevice, Interface, SpiInterface),
    sync(feature = "sync"),
    async(feature = "async")
)]
impl<SPI: SpiDevice> Interface for SpiInterface<SPI> {
    type Error = SPI::Error;

    async fn write_register(
        &mut self,
        c: Register,
        data: [u8; 2],
    ) -> Result<(), DacError<SPI::Error>> {
        self.spi
            .write(&[c as u8, data[0], data[1]])
            .await
            .map_err(DacError::InterfaceError)
    }
}

/// Wrapper over an I2C interface, holding the embedded-hal i2c data and the device address
#[maybe_async_cfg::maybe(idents(I2cInterface), sync(feature = "sync"), async(feature = "async"))]
pub struct I2cInterface<I2C> {
    i2c: I2C,
    address: u8,
}
#[maybe_async_cfg::maybe(
    idents(I2c, Interface, I2cInterface),
    sync(feature = "sync"),
    async(feature = "async")
)]
impl<I2C: I2c> Interface for I2cInterface<I2C> {
    type Error = I2C::Error;

    async fn write_register(
        &mut self,
        c: Register,
        data: [u8; 2],
    ) -> Result<(), DacError<Self::Error>> {
        self.i2c
            .write(self.address, &[c as u8, data[0], data[1]])
            .await
            .map_err(DacError::InterfaceError)
    }
}

/// Generic DAC. Use [`Dac80501`], [`Dac70501`] or [`Dac60501`] rather than instantiating this
#[maybe_async_cfg::maybe(idents(Dac), sync(feature = "sync"), async(feature = "async"))]
pub struct Dac<I, const BITS: u8> {
    interface: I,
    config: DacConfig,
}
#[maybe_async_cfg::maybe(
    idents(SpiDevice, Dac, SpiInterface),
    sync(feature = "sync"),
    async(feature = "async")
)]
impl<SPI: SpiDevice, const BITS: u8> Dac<SpiInterface<SPI>, BITS> {
    /// Creates a new instance of the specified dac with the internal state set to match
    /// the device defaults using an SPI interface
    pub fn new_spi(spi: SPI) -> Self {
        const {
            assert!(
                BITS == 12 || BITS == 14 || BITS == 16,
                "BITS must be 12, 14, or 16"
            )
        };
        Self {
            interface: SpiInterface { spi },
            config: DacConfig::default(),
        }
    }
}

#[maybe_async_cfg::maybe(
    idents(I2c, Dac, I2cInterface),
    sync(feature = "sync"),
    async(feature = "async")
)]
impl<I2C: I2c, const BITS: u8> Dac<I2cInterface<I2C>, BITS> {
    /// Creates a new instance of the specified dac with the internal state set to match
    /// the device defaults using an I2C interface
    pub fn new_i2c(i2c: I2C, address: u8) -> Self {
        const {
            assert!(
                BITS == 12 || BITS == 14 || BITS == 16,
                "BITS must be 12, 14, or 16"
            )
        };
        Self {
            interface: I2cInterface { i2c, address },
            config: DacConfig::default(),
        }
    }

    async fn read_register(&mut self, c: Register) -> Result<[u8; 2], DacError<I2C::Error>> {
        let mut buf = [0u8; 2];
        self.interface
            .i2c
            .write_read(self.interface.address, &[c as u8], &mut buf)
            .await
            .map_err(DacError::InterfaceError)?;
        Ok(buf)
    }

    /// Returns the resolution of the device in bits
    pub async fn resolution(&mut self) -> Result<u8, DacError<I2C::Error>> {
        let buf = self.read_register(Register::DEVID).await?;
        match buf[0] >> 4 {
            0b000 => Ok(16),
            0b001 => Ok(14),
            0b010 => Ok(12),
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns the power on [`ResetValue`] of the DAC
    pub async fn reset_value(&mut self) -> Result<ResetValue, DacError<I2C::Error>> {
        let buf = self.read_register(Register::DEVID).await?;
        match buf[1] >> 7 {
            0b00 => Ok(ResetValue::Zero),
            0b01 => Ok(ResetValue::MidScale),
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns whether the device is in synchronous or asynchronous [`UpdateMode`]
    pub async fn update_mode(&mut self) -> Result<UpdateMode, DacError<I2C::Error>> {
        let buf = self.read_register(Register::SYNC).await?;
        match buf[1] & 0b0000_0001 {
            0b0 => Ok(UpdateMode::Asynchronous),
            0b1 => Ok(UpdateMode::Synchronous),
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns the [`InternalReference`] state
    pub async fn internal_reference(&mut self) -> Result<InternalReference, DacError<I2C::Error>> {
        let buf = self.read_register(Register::CONFIG).await?;
        match buf[0] & 0b0000_0001 {
            0b0 => {
                self.config.ref_power = InternalReference::Enabled;
                Ok(InternalReference::Enabled)
            }
            0b1 => {
                self.config.ref_power = InternalReference::Enabled;
                Ok(InternalReference::Disabled)
            }
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns the DAC [`PowerState`]
    pub async fn power_state(&mut self) -> Result<PowerState, DacError<I2C::Error>> {
        let buf = self.read_register(Register::CONFIG).await?;
        match buf[1] & 0b0000_0001 {
            0b0 => {
                self.config.dac_power = PowerState::On;
                Ok(PowerState::On)
            }
            0b1 => {
                self.config.dac_power = PowerState::Down;
                Ok(PowerState::Down)
            }
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns the status of the [`ReferenceDivider`]
    pub async fn reference_divider(&mut self) -> Result<ReferenceDivider, DacError<I2C::Error>> {
        let buf = self.read_register(Register::GAIN).await?;
        match buf[0] & 0b0000_0001 {
            0b0 => Ok(ReferenceDivider::None),
            0b1 => {
                self.config.ref_divider = ReferenceDivider::Two;
                Ok(ReferenceDivider::Two)
            }
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns the output [`BufferGain`]
    pub async fn output_gain(&mut self) -> Result<BufferGain, DacError<I2C::Error>> {
        let buf = self.read_register(Register::GAIN).await?;
        match buf[1] & 0b0000_0001 {
            0b0 => {
                self.config.buffer_gain = BufferGain::None;
                Ok(BufferGain::None)
            }
            0b1 => {
                self.config.buffer_gain = BufferGain::Two;
                Ok(BufferGain::Two)
            }
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns reference [`AlarmStatus`]
    pub async fn alarm_status(&mut self) -> Result<AlarmStatus, DacError<I2C::Error>> {
        let buf = self.read_register(Register::STATUS).await?;
        match buf[1] & 0b0000_0001 {
            0b0 => Ok(AlarmStatus::Normal),
            0b1 => Ok(AlarmStatus::Alarm),
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns the current output level of the DAC
    pub async fn output_level(&mut self) -> Result<u16, DacError<I2C::Error>> {
        let buf = self.read_register(Register::DACDATA).await?;
        Ok(u16::from_be_bytes(buf) >> (Self::REGISTER_WIDTH - BITS))
    }
}

#[maybe_async_cfg::maybe(
    idents(Dac, Interface),
    sync(feature = "sync"),
    async(feature = "async")
)]
impl<I, const BITS: u8> Dac<I, BITS>
where
    I: Interface,
{
    /// Width of the DACDATA shift register in bits. Fixed by the hardware
    /// regardless of the DAC resolution. 12 and 14 bit DACs left justify output code within output register.
    const REGISTER_WIDTH: u8 = 16;

    /// Write to the NOOP register, has no effect
    pub async fn noop(&mut self) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(Register::NOOP, [0x00, 0x00])
            .await
    }

    /// Set whether the DAC trigger [`UpdateMode`]
    pub async fn set_update_mode(&mut self, mode: UpdateMode) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(Register::SYNC, [0x00, mode as u8])
            .await
    }

    /// Enables and disables the device [`InternalReference`]
    pub async fn set_internal_reference(
        &mut self,
        intern_ref: InternalReference,
    ) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(
                Register::CONFIG,
                [intern_ref as u8, self.config.dac_power as u8],
            )
            .await?;
        self.config.ref_power = intern_ref;
        Ok(())
    }

    /// Set the device [`PowerState`]
    pub async fn set_power_state(&mut self, state: PowerState) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(Register::CONFIG, [self.config.ref_power as u8, state as u8])
            .await?;
        self.config.dac_power = state;
        Ok(())
    }

    /// Set the [`ReferenceDivider`]
    pub async fn set_reference_divider(
        &mut self,
        ref_div: ReferenceDivider,
    ) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(
                Register::GAIN,
                [ref_div as u8, self.config.buffer_gain as u8],
            )
            .await?;
        self.config.ref_divider = ref_div;
        Ok(())
    }

    /// Set the [`BufferGain`]
    pub async fn set_output_gain(&mut self, gain: BufferGain) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(Register::GAIN, [self.config.ref_divider as u8, gain as u8])
            .await?;
        self.config.buffer_gain = gain;
        Ok(())
    }

    /// Trigger synchronous load. Self resetting after load is completed. No effect for asynchronous operation.
    pub async fn load_dac(&mut self) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(Register::TRIGGER, [0x00, 0b0001_0000])
            .await
    }

    /// Soft reset, reset device to power on defaults.
    pub async fn soft_reset(&mut self) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(Register::TRIGGER, [0x00, 0b0000_1010])
            .await?;
        self.config = DacConfig::default();
        Ok(())
    }

    /// Set the output voltage of the device and check the bounds on number of bits
    pub async fn set_output_level(&mut self, level: u16) -> Result<(), DacError<I::Error>> {
        // Shifts to ensure level is not out of range for the number of bits the DAC has.
        // Check should be optimized out in the case of a 16bit DAC
        if level.checked_shr(BITS as u32).unwrap_or(0) != 0 {
            return Err(DacError::ValueOverflow);
        }

        let bytes: [u8; 2] = (level << (Self::REGISTER_WIDTH - BITS)).to_be_bytes();
        self.interface
            .write_register(Register::DACDATA, bytes)
            .await
    }

    /// Sets the output level without checking the number of bits being set
    pub async fn set_output_level_unchecked(
        &mut self,
        level: u16,
    ) -> Result<(), DacError<I::Error>> {
        let bytes: [u8; 2] = (level << (Self::REGISTER_WIDTH - BITS)).to_be_bytes();
        self.interface
            .write_register(Register::DACDATA, bytes)
            .await
    }
}

#[cfg(feature = "sync")]
/// DAC80501, 16bit dac, synchronous
pub type Dac80501<I> = DacSync<I, 16>;
#[cfg(feature = "sync")]
/// DAC70501, 14bit dac, synchronous
pub type Dac70501<I> = DacSync<I, 14>;
#[cfg(feature = "sync")]
/// DAC60501, 12bit dac, synchronous
pub type Dac60501<I> = DacSync<I, 12>;

#[cfg(feature = "async")]
/// DAC80501, 16bit dac, asynchronous
pub type AsyncDac80501<I> = DacAsync<I, 16>;
#[cfg(feature = "async")]
/// DAC70501, 14bit dac, asynchronous
pub type AsyncDac70501<I> = DacAsync<I, 14>;
#[cfg(feature = "async")]
/// DAC60501, 12bit dac, asynchronous
pub type AsyncDac60501<I> = DacAsync<I, 12>;


use embedded_hal_async::i2c::I2c;
use embedded_hal_async::spi::SpiDevice;

use crate::{
    AlarmStatus, BufferGain, DacConfig, DacError, InternalReference, Mode, PowerState,
    ReferenceDivider, Register, ResetValue,
};

// Generic interface implementation. Allows the device to be setup using either SPI or I2C with the
/// same API.
/// Uses `async fn` in a public trait. Intended for use with single threaded async executors (embassy)
/// Therefore no `Send` bound is needed on the futures.
#[allow(async_fn_in_trait)]
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
pub struct SpiInterface<SPI> {
    spi: SPI,
}
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
pub struct I2cInterface<I2C> {
    i2c: I2C,
    address: u8,
}
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

/// Generic DAC. Use [`Dac80501`], [`Dac70501`] or [`Dac60501`] for the specific DACs rather than instantiating this directly
pub struct Dac<I, const BITS: u8> {
    interface: I,
    config: DacConfig,
}

impl<SPI: SpiDevice, const BITS: u8> Dac<SpiInterface<SPI>, BITS> {
    /// Creates a new instance of the specified dac with the internal state set to match
    /// the device defaults using an SPI interface
    pub fn new_spi(spi: SPI) -> Self {
        Self {
            interface: SpiInterface { spi },
            config: DacConfig::default(),
        }
    }
}

/// Dac80501, 16 bit DAC,
pub type Dac80501<I> = Dac<I, 16>;
/// Dac70501, 14 bit DAC
pub type Dac70501<I> = Dac<I, 14>;
/// Dac60501, 12 bit DAC
pub type Dac60501<I> = Dac<I, 12>;

impl<I2C: I2c, const BITS: u8> Dac<I2cInterface<I2C>, BITS> {
    /// Creates a new instance of the specified dac with the internal state set to match
    /// the device defaults using an I2C interface
    pub fn new_i2c(i2c: I2C, address: u8) -> Self {
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
    pub async fn get_resolution(&mut self) -> Result<u8, DacError<I2C::Error>> {
        let buf = self.read_register(Register::DEVID).await?;
        match buf[0] >> 4 {
            0b000 => Ok(16),
            0b001 => Ok(14),
            0b010 => Ok(12),
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns the power on reset value of the DAC
    pub async fn get_reset_value(&mut self) -> Result<ResetValue, DacError<I2C::Error>> {
        let buf = self.read_register(Register::DEVID).await?;
        match buf[1] >> 7 {
            0b00 => Ok(ResetValue::Zero),
            0b01 => Ok(ResetValue::MidScale),
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns whether the device is in synchronous or asynchronous mode
    pub async fn get_synchronous(&mut self) -> Result<Mode, DacError<I2C::Error>> {
        let buf = self.read_register(Register::SYNC).await?;
        match buf[1] & 0b0000000_1 {
            0b0 => Ok(Mode::Asynchronous),
            0b1 => Ok(Mode::Synchronous),
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns the whether the internal reference is enabled or disabled
    pub async fn get_internal_reference(
        &mut self,
    ) -> Result<InternalReference, DacError<I2C::Error>> {
        let buf = self.read_register(Register::CONFIG).await?;
        match buf[0] & 0b0000000_1 {
            0b0 => Ok(InternalReference::Enabled),
            0b1 => Ok(InternalReference::Disabled),
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns the DAC power state, on or down
    pub async fn get_power_state(&mut self) -> Result<PowerState, DacError<I2C::Error>> {
        let buf = self.read_register(Register::CONFIG).await?;
        match buf[1] & 0b0000000_1 {
            0b0 => Ok(PowerState::On),
            0b1 => Ok(PowerState::Down),
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns the status of the reference divider, either no reference division or divide by two
    pub async fn get_reference_divider(
        &mut self,
    ) -> Result<ReferenceDivider, DacError<I2C::Error>> {
        let buf = self.read_register(Register::GAIN).await?;
        match buf[0] & 0b0000000_1 {
            0b0 => Ok(ReferenceDivider::None),
            0b1 => Ok(ReferenceDivider::Two),
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns the output buffer gain either no gain or two times gain
    pub async fn get_output_gain(&mut self) -> Result<BufferGain, DacError<I2C::Error>> {
        let buf = self.read_register(Register::GAIN).await?;
        match buf[1] & 0b0000000_1 {
            0b0 => Ok(BufferGain::None),
            0b1 => Ok(BufferGain::Two),
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns reference alarm status. Alarm occurs when supply is below what is required to output the maximum output voltage.
    pub async fn get_alarm_status(&mut self) -> Result<AlarmStatus, DacError<I2C::Error>> {
        let buf = self.read_register(Register::STATUS).await?;
        match buf[1] & 0b0000000_1 {
            0b0 => Ok(AlarmStatus::Normal),
            0b1 => Ok(AlarmStatus::Alarm),
            _ => Err(DacError::UnknownValue),
        }
    }

    /// Returns the current output level of the DAC
    pub async fn get_output_level(&mut self) -> Result<u16, DacError<I2C::Error>> {
        let buf = self.read_register(Register::DACDATA).await?;
        Ok(u16::from_be_bytes(buf) >> (Self::REGISTER_WIDTH - BITS))
    }
}

impl<I, const BITS: u8> Dac<I, BITS>
where
    I: Interface,
{
    /// Width of the DACDATA shift register in bits. Fixed by the hardware
    /// regardless of the DAC resolution. 12 and 14 bit DACs left justify output code within output register.
    const REGISTER_WIDTH: u8 = 16;

    /// Write to the NOOP register, has no effects
    pub async fn set_noop(&mut self) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(Register::NOOP, [0x00, 0x00])
            .await
    }

    /// Set whether the DAC is triggered by load DAC or if it is set to update immediately
    pub async fn set_synchronous(&mut self, mode: Mode) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(Register::SYNC, [0x00, mode as u8])
            .await
    }

    /// Enables and disables the device internal reference. The internal reference is on by default
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

    /// In power-off state the device output is connected to GND through a 1-kΩ internal
    /// resistor. The device is in power `On` state by default. This reduces current
    /// consumption to typically 15 µA at 5 V.
    pub async fn set_power_state(&mut self, state: PowerState) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(Register::CONFIG, [self.config.ref_power as u8, state as u8])
            .await?;
        self.config.dac_power = state;
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

    /// When set to `TwoX`, the buffer amplifier for the DAC has a gain of 2x doubling the
    /// voltage output. When set to `OneX` it has a gain of 1x. Using this gain can be
    /// especially useful when using the internal reference divider set to `Half`. The
    /// output gain is set to `TwoX` by default
    pub async fn set_output_gain(&mut self, gain: BufferGain) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(Register::GAIN, [self.config.ref_divider as u8, gain as u8])
            .await?;
        self.config.buffer_gain = gain;
        Ok(())
    }

    /// Trigger synchronous load. Self resetting after load is completed. No effect for asynchronous operation.
    pub async fn set_load_dac(&mut self) -> Result<(), DacError<I::Error>> {
        self.interface
            .write_register(Register::TRIGGER, [0x00, 0b000_1_0000])
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

    /// Set the output voltage of the device and check the level bounds for the specified device
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

    /// This function sets the output level without checking the bounds on the size of the
    /// value for the specified DAC
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

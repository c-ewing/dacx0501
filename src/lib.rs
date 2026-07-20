//! This crate is an embedded-hal driver library implementation for the Texas Instruments 80501,
//! 70501 and 60501 DACs. It relies on the embedded-hal 1.0.0 traits being implemented in
//! the board hal. See the [product page](https://www.ti.com/product/DAC80501/part-details/DAC80501ZDQFT) for the datasheet and other notes.

#![no_std]
#![deny(missing_docs)]
#![doc(
    html_logo_url = "https://www.ti.com/content/dam/ticom/images/products/package/d/dqf0008a.png"
)]

use core::convert::Infallible;
use core::fmt;

use embedded_hal::spi::SpiDevice;

/// Command byte, first byte of the transfer to the DAC
///
/// B23 B22 B21 B20 B19 B18 B17 B16 REGISTER     HEX
///  0   0   0   0   0   0   0   0   NOOP        0x00
///  0   0   0   0   0   0   0   1   DEVID       0x01
///  0   0   0   0   0   0   1   1   SYNC        0x02
///  0   0   0   0   0   0   1   1   CONFIG      0x03
///  0   0   0   0   0   1   0   0   GAIN        0x04
///  0   0   0   0   0   1   0   1   TRIGGER     0x05
///  0   0   0   0   0   1   1   1   STATUS      0x07
///  0   0   0   0   1   0   0   0   DACDATA     0x08
#[allow(dead_code)]
#[repr(C)]
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

#[derive(Default)]
struct DacState {
    config: DacConfig,
    gain: GainConfig,
}

#[derive(Default)]
struct DacConfig {
    ref_pwdwn: InternalReference,
    dac_pwdwn: PowerState,
}
impl DacConfig {
    fn to_array(&self) -> [u8; 2] {
        [
            // When set to 1, this bit disables the device internal reference.
            matches!(self.ref_pwdwn, InternalReference::Disabled) as u8,
            // When set to 1, the DAC in power-down mode and the DAC output is connected to GND
            // through a 1-kΩ internal resistor.
            matches!(self.dac_pwdwn, PowerState::Down) as u8,
        ]
    }
}

struct GainConfig {
    ref_div: ReferenceDivider,
    buff_gain: BufferGain,
}
impl Default for GainConfig {
    fn default() -> Self {
        Self {
            ref_div: ReferenceDivider::None,
            buff_gain: BufferGain::Two,
        }
    }
}
impl GainConfig {
    fn to_array(&self) -> [u8; 2] {
        [
            // When REF-DIV set to 1, the reference voltage is internally divided by a factor of 2.
            matches!(self.ref_div, ReferenceDivider::Two) as u8,
            // When set to 1, the buffer amplifier for corresponding DAC has a gain of 2.
            matches!(self.buff_gain, BufferGain::Two) as u8,
        ]
    }
}

/// DAC power state. When powered down the DAC output is connected to ground through a 1k resistor.
/// The device default is [`PowerState::On`]
pub enum PowerState {
    /// Normal operation
    On,
    /// Power down, output connected to ground
    Down,
}
impl Default for PowerState {
    fn default() -> Self {
        Self::On
    }
}

/// Output buffer gain.
/// Power on value is [`BufferGain::Two`]
pub enum BufferGain {
    /// The output voltage of the device is [0 .. VREF]
    None,
    /// The output voltage of the device is [0 .. 2*VREF]
    Two,
}
impl Default for BufferGain {
    fn default() -> Self {
        Self::Two
    }
}

/// DAC reference divider which applies to both internal and external reference sources.
/// Power on value is [`ReferenceDivider::None`]
pub enum ReferenceDivider {
    /// The reference voltage is not modified
    None,
    /// The reference voltage is divided by 2
    Two,
}
impl Default for ReferenceDivider {
    fn default() -> Self {
        Self::None
    }
}

/// Status of the internal reference.
/// Power on value is [`InternRefState::Enable`]
pub enum InternalReference {
    /// The device internal reference is enabled
    Enabled,
    /// The device internal reference is disabled. External reference must be provided.
    Disabled,
}
impl Default for InternalReference {
    fn default() -> Self {
        Self::Enabled
    }
}

#[derive(PartialEq, Eq)]
/// Alarm when supply voltage is below what is required to power the internal reference and gain buffer. DAC outputs 0 volts while supply is too low.
/// Upon supply exceeding the analog threshold DAC output returns to normal operation with the output code uneffected.
/// Power on value is [`AlarmStatus::Normal`]
pub enum AlarmStatus {
    /// Not enough headroom, reference buffer shutdown. DAC outputs 0 volts.
    Alarm,
    /// Normal operation
    Normal,
}

#[derive(Debug)]
/// The custom error for this crate
pub enum DacError {
    /// The value for the specified DAC overflowed
    ValueOverflow,
    /// An internal embedded hal SPI transfer error
    SpiError,
}
impl fmt::Display for DacError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValueOverflow => f.write_str("The data value was too large for the selected DAC"),
            Self::SpiError => f.write_str("Internal HAL SPI error"),
        }
    }
}
impl From<embedded_hal::spi::ErrorKind> for DacError {
    fn from(_: embedded_hal::spi::ErrorKind) -> Self {
        DacError::SpiError
    }
}

impl From<Infallible> for DacError {
    fn from(_: Infallible) -> Self {
        DacError::SpiError
    }
}

macro_rules! Dac {
    ($(#[$meta:meta])* $Name:ident, $bits:expr) => {

        impl<SPI> $Name<SPI>
        where
            SPI: SpiDevice,
            DacError: core::convert::From<<SPI as embedded_hal::spi::ErrorType>::Error>,
        {
            /// Set the output voltage of the device and check the level bounds for the specified device
            pub fn set_output_level(&mut self, level: u16) -> Result<(), DacError> {
                // Data are MSB aligned in straight binary format
                if level as u32 & (1u32 << $bits) > 0 {
                    return Err(DacError::ValueOverflow);
                }
                self.data[0] = Command::DACDATA as u8;
                self.data[1..].copy_from_slice(level.to_be_bytes().as_slice());
                self.spi.write(&self.data).map_err(DacError::from)?;
                Ok(())
            }

            /// # Safety
            ///
            /// This function sets the output level without checking the bounds on the size of the
            /// value for the specified DAC
            pub unsafe fn set_output_level_unckecked(&mut self, level: u16) -> Result<(), DacError> {
                // Data are MSB aligned in straight binary format
                self.data[0] = Command::DACDATA as u8;
                self.data[1..].copy_from_slice(level.to_be_bytes().as_slice());
                self.spi.write(&self.data).map_err(DacError::from)?;
                Ok(())
            }

        }

        Dac!($(#[$meta])* $Name :! dc);

    };

    ($(#[$meta:meta])* $Name:ident) => {

        impl<SPI> $Name<SPI>
        where
            SPI: SpiDevice,
            DacError: core::convert::From<<SPI as embedded_hal::spi::ErrorType>::Error>,
        {
            /// Set the output voltage of the device without any extra bounds checks
            pub fn set_output_level(&mut self, level: u16) -> Result<(), DacError> {
                // Data are MSB aligned in straight binary format
                self.data[0] = Command::DACDATA as u8;
                self.data[1..].copy_from_slice(level.to_be_bytes().as_slice());
                self.spi.write(&self.data).map_err(DacError::from)?;
                Ok(())
            }
        }

        Dac!($(#[$meta])* $Name :! dc);

    };

    ($(#[$meta:meta])* $Name:ident :! $DC:ident) => {

        $(#[$meta])*
        pub struct $Name<SPI> {
            spi: SPI,
            data: [u8; 3],
            dac_state: DacState,
        }

        impl<SPI> $Name<SPI>
        where
            SPI: SpiDevice,
            DacError: core::convert::From<<SPI as embedded_hal::spi::ErrorType>::Error>,
        {
            /// Creates a new instance of the specified dac with the internal state set to match
            /// the device defaults
            pub fn new(spi: SPI) -> Self {
                Self {
                    spi,
                    data: [0, 0, 0],
                    dac_state: DacState::default(),
                }
            }


            /// Enables and disables the device internal reference. The internal reference is on by default
            pub fn set_internal_reference(
                &mut self,
                intern_ref: InternalReference,
            ) -> Result<(), DacError> {
                self.dac_state.config.ref_pwdwn = intern_ref;
                self.data[0] = Command::CONFIG as u8;
                self.data[1..].copy_from_slice(&self.dac_state.config.to_array());
                self.spi.write(&self.data).map_err(DacError::from)?;
                Ok(())
            }

            /// In power-off state the device output is connected to GND through a 1-kΩ internal
            /// resistor. The device is in power `On` state by default. This reduces current
            /// consumption to typically 15 µA at 5 V.
            pub fn set_power_state(&mut self, state: PowerState) -> Result<(), DacError> {
                self.dac_state.config.dac_pwdwn = state;
                self.data[0] = Command::CONFIG as u8;
                self.data[1..].copy_from_slice(&self.dac_state.config.to_array());
                self.spi.write(&self.data).map_err(DacError::from)?;
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
            pub fn set_reference_divider(&mut self, ref_div: ReferenceDivider) -> Result<(), DacError> {
                self.dac_state.gain.ref_div = ref_div;
                self.data[0] = Command::GAIN as u8;
                self.data[1..].copy_from_slice(&self.dac_state.gain.to_array());
                self.spi.write(&self.data).map_err(DacError::from)?;
                Ok(())
            }

            /// When set to `TwoX`, the buffer amplifier for the DAC has a gain of 2x doubling the
            /// voltage output. When set to `OneX` it has a gain of 1x. Using this gain can be
            /// especially useful when using the internal reference divider set to `Half`. The
            /// output gain is set to `TwoX` by default
            pub fn set_output_gain(&mut self, gain: BufferGain) -> Result<(), DacError> {
                self.dac_state.gain.buff_gain = gain;
                self.data[0] = Command::GAIN as u8;
                self.data[1..].copy_from_slice(&self.dac_state.gain.to_array());
                self.spi.write(&self.data).map_err(DacError::from)?;
                Ok(())
            }
        }

        impl<SPI> $Name<SPI>
        where
            SPI: SpiDevice,
            DacError: core::convert::From<<SPI as embedded_hal::spi::ErrorType>::Error>,
        {
            /// `AlarmStatus` is `High` when the difference between the reference and supply pins is below a minimum
            /// analog threshold. The status is `Low` otherwise. When `High`, the reference buffer is shut down, and the DAC
            /// outputs are all zero volts. The DAC codes are unaffected, and the DAC output returns to
            /// normal when the difference is above the analog threshold.
            pub fn ref_alarm_status(&mut self) -> Result<AlarmStatus, DacError> {
                self.data[0] = Command::STATUS as u8;
                self.data[1] = 0;
                self.data[2] = 0;
                self.spi.read(&mut self.data).map_err(DacError::from)?;
                if self.data[2] == 1 {
                    Ok(AlarmStatus::Alarm)
                } else {
                    Ok(AlarmStatus::Normal)
                }
            }
        }
    };
}

Dac!(
    /// A 16 bit DAC
    Dac80501
);
Dac!(
    /// A 14 bit DAC
    Dac70501,
    14
);
Dac!(
    /// A 12 bit DAC
    Dac60501,
    12
);

use std::assert_matches;
use std::u16;

use dacx0501::BufferGain;
use dacx0501::InternalReference;
use dacx0501::Mode;
use dacx0501::PowerState;
use dacx0501::ReferenceDivider;
use dacx0501::{self};
use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};

#[test]
fn construction_16() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let _d16 = dacx0501::Dac80501::new(&mut spi);

    spi.done();
}

#[test]
fn construction_14() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let _d14 = dacx0501::Dac70501::new(&mut spi);

    spi.done();
}

#[test]
fn construction_12() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let _d12 = dacx0501::Dac60501::new(&mut spi);

    spi.done();
}

// NOOP Register
#[test]
fn set_noop() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x00, 0x00, 0x00]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);

    d12.set_noop().expect("Writing to noop should not panic");
    spi.done();
}

// DEVID Register
#[test]
#[should_panic]
fn read_resolution() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);

    // Unimplemented for SPI
    let _ = d12.get_resolution();

    spi.done();
}

// Sync Register
#[test]
#[should_panic]
fn read_sync() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);

    // Unimplemented for SPI
    let _ = d12.get_synchronous();

    spi.done();
}

#[test]
fn set_sync() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x02, 0x00, 0x00]),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x02, 0x00, 0x01]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);

    d12.set_synchronous(Mode::Asynchronous)
        .expect("Should not panic setting async");
    d12.set_synchronous(Mode::Synchronous)
        .expect("Should not panic setting sync");
    spi.done();
}

// Config Register
#[test]
#[should_panic]
fn read_reference() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);

    // Unimplemented for SPI
    let _ = d12.get_internal_reference();

    spi.done();
}

#[test]
fn set_reference() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x03, 0b0000000_0, 0x00]),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x03, 0b0000000_1, 0x00]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);
    d12.set_internal_reference(InternalReference::Enabled)
        .expect("Shouldn't panic on turning reference on");

    d12.set_internal_reference(InternalReference::Disabled)
        .expect("Shouldn't panic on turning reference off");

    spi.done();
}

#[test]
#[should_panic]
fn read_power_state() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);

    // Unimplemented for SPI
    let _ = d12.get_power_state();

    spi.done();
}

#[test]
fn set_powerdown() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x03, 0x00, 0b0000000_0]),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x03, 0x00, 0b0000000_1]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);
    d12.set_power_state(PowerState::On)
        .expect("Shouldn't panic on turning dac on");

    d12.set_power_state(PowerState::Down)
        .expect("Shouldn't panic on turning dac off");

    spi.done();
}

// GAIN Register
#[test]
#[should_panic]
fn read_reference_divider() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);

    // Unimplemented for SPI
    let _ = d12.get_reference_divider();

    spi.done();
}

#[test]
fn set_reference_divider() {
    // NOTE: Default value of BUFF-GAIN bit is 1
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x04, 0b0000000_0, 0x01]),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x04, 0b0000000_1, 0x01]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);
    d12.set_reference_divider(ReferenceDivider::None)
        .expect("Shouldn't panic on changing reference divider");

    d12.set_reference_divider(ReferenceDivider::Two)
        .expect("Shouldn't panic on changing reference divider");

    spi.done();
}

#[test]
#[should_panic]
fn read_buffer_gain() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);

    // Unimplemented for SPI
    let _ = d12.get_output_gain();

    spi.done();
}

#[test]
fn set_buffer_gain() {
    // NOTE: Default value of BUFF-GAIN bit is 1
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x04, 0x00, 0b0000000_0]),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x04, 0x00, 0b0000000_1]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);
    d12.set_output_gain(BufferGain::None)
        .expect("Shouldn't panic on changing buffer gain");

    d12.set_output_gain(BufferGain::Two)
        .expect("Shouldn't panic on changing buffer gain");

    spi.done();
}

// TRIGGER Register
#[test]
fn set_load_dac() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x05, 0x00, 0b000_1_0000]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);

    d12.set_load_dac()
        .expect("Triggering load should not panic");
    spi.done();
}

#[test]
fn set_soft_reset() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x05, 0x00, 0b000_1010]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut _d12 = dacx0501::Dac60501::new(&mut spi);

    unimplemented!("Soft reset not supported");
}

// STATUS Register
// TODO: SPI does not support reads

// DAC Register
#[test]
fn set_output_0() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x08, 0x00, 0x00]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);
    d12.set_output_level(0 as u16)
        .expect("Shouldn't panic on setting dac to 0 output");

    spi.done();
}

#[test]
fn set_output_max_err() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);

    assert_matches!(
        d12.set_output_level(u16::MAX),
        Err(dacx0501::DacError::ValueOverflow)
    );

    spi.done();
}

#[test]
fn set_output_max() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x08, 0xFF, 0xFF]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d16 = dacx0501::Dac80501::new(&mut spi);

    assert_matches!(d16.set_output_level(u16::MAX), Ok(()));

    spi.done();
}

#[test]
fn set_output_mid_scale() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![0x08, 0x08, 0x00]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);

    d12.set_output_level(2048)
        .expect("Setting to mid scale should not panic");

    spi.done();
}

#[test]
#[should_panic]
fn read_alarm() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new(&mut spi);

    // Unimplemented for SPI
    let _ = d12.ref_alarm_status();

    spi.done();
}

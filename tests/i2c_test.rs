use std::assert_matches;
use std::u16;

use dacx0501::AlarmStatus;
use dacx0501::BufferGain;
use dacx0501::InternalReference;
use dacx0501::Mode;
use dacx0501::PowerState;
use dacx0501::ReferenceDivider;
use dacx0501::ResetValue;
use dacx0501::{self};
use embedded_hal_mock::eh1::i2c::{Mock as I2cMock, Transaction as I2cTransaction};

const ADDR: u8 = 0b1001_000;

#[test]
fn construction_16() {
    let expectations = [];
    let mut i2c = I2cMock::new(&expectations);
    let _d16 = dacx0501::Dac80501::new_i2c(&mut i2c, ADDR);

    i2c.done();
}

#[test]
fn construction_14() {
    let expectations = [];
    let mut i2c = I2cMock::new(&expectations);
    let _d14 = dacx0501::Dac70501::new_i2c(&mut i2c, ADDR);

    i2c.done();
}

#[test]
fn construction_12() {
    let expectations = [];
    let mut i2c = I2cMock::new(&expectations);
    let _d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    i2c.done();
}

// NOOP Register
#[test]
fn set_noop() {
    let expectations = [I2cTransaction::write(ADDR, vec![0x00, 0x00, 0x00])];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    d12.set_noop().expect("Writing to noop should not panic");
    i2c.done();
}

// DEVID Register
#[test]
fn read_resolution() {
    let expectations = [
        I2cTransaction::write_read(ADDR, vec![0b0000_0001 as u8], vec![0b0_000_0001, 0x00]),
        I2cTransaction::write_read(ADDR, vec![0b0000_0001 as u8], vec![0b0_001_0001, 0x00]),
        I2cTransaction::write_read(ADDR, vec![0b0000_0001 as u8], vec![0b0_010_0001, 0x00]),
    ];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    let r = d12
        .get_resolution()
        .expect("Reading resolution should not panic");
    assert_eq!(r, 16);

    let r = d12
        .get_resolution()
        .expect("Reading resolution should not panic");
    assert_eq!(r, 14);

    let r = d12
        .get_resolution()
        .expect("Reading resolution should not panic");
    assert_eq!(r, 12);

    i2c.done();
}

#[test]
fn read_reset_value() {
    let expectations = [
        I2cTransaction::write_read(ADDR, vec![0b0000_0001 as u8], vec![0x00, 0b0_0010101]),
        I2cTransaction::write_read(ADDR, vec![0b0000_0001 as u8], vec![0x00, 0b1_0010101]),
    ];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    let reset = d12
        .get_reset_value()
        .expect("Reading reset value should not panic");
    assert_eq!(reset, ResetValue::Zero);

    let reset = d12
        .get_reset_value()
        .expect("Reading reset value should not panic");
    assert_eq!(reset, ResetValue::MidScale);

    i2c.done();
}

// Sync Register
#[test]
fn read_sync() {
    let expectations = [
        I2cTransaction::write_read(ADDR, vec![0b0000_0010 as u8], vec![0x00, 0x00]),
        I2cTransaction::write_read(ADDR, vec![0b0000_0010 as u8], vec![0x00, 0x01]),
    ];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    let sync = d12.get_synchronous().expect("Should not panic");
    assert_eq!(sync, Mode::Asynchronous);

    let sync = d12.get_synchronous().expect("Should not panic");
    assert_eq!(sync, Mode::Synchronous);

    i2c.done();
}

#[test]
fn set_sync() {
    let expectations = [
        I2cTransaction::write(ADDR, vec![0x02, 0x00, 0x00]),
        I2cTransaction::write(ADDR, vec![0x02, 0x00, 0x01]),
    ];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    d12.set_synchronous(Mode::Asynchronous)
        .expect("Should not panic setting async");
    d12.set_synchronous(Mode::Synchronous)
        .expect("Should not panic setting sync");
    i2c.done();
}

// Config Register
#[test]
fn read_reference() {
    let expectations = [
        I2cTransaction::write_read(ADDR, vec![0b0000_0011 as u8], vec![0x00, 0x00]),
        I2cTransaction::write_read(ADDR, vec![0b0000_0011 as u8], vec![0x01, 0x00]),
    ];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    let reference = d12
        .get_internal_reference()
        .expect("Reading power should not panic");
    assert_eq!(reference, InternalReference::Enabled);

    let reference = d12
        .get_internal_reference()
        .expect("Reading power should not panic");
    assert_eq!(reference, InternalReference::Disabled);

    i2c.done();
}

#[test]
fn set_reference() {
    let expectations = [
        I2cTransaction::write(ADDR, vec![0x03, 0b0000000_0, 0x00]),
        I2cTransaction::write(ADDR, vec![0x03, 0b0000000_1, 0x00]),
    ];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);
    d12.set_internal_reference(InternalReference::Enabled)
        .expect("Shouldn't panic on turning reference on");

    d12.set_internal_reference(InternalReference::Disabled)
        .expect("Shouldn't panic on turning reference off");

    i2c.done();
}

#[test]
fn read_power_state() {
    let expectations = [
        I2cTransaction::write_read(ADDR, vec![0b0000_0011 as u8], vec![0x00, 0x00]),
        I2cTransaction::write_read(ADDR, vec![0b0000_0011 as u8], vec![0x00, 0x01]),
    ];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    let power = d12
        .get_power_state()
        .expect("Reading power should not panic");
    assert_eq!(power, PowerState::On);

    let power = d12
        .get_power_state()
        .expect("Reading power should not panic");
    assert_eq!(power, PowerState::Down);

    i2c.done();
}

#[test]
fn set_powerdown() {
    let expectations = [
        I2cTransaction::write(ADDR, vec![0x03, 0x00, 0b0000000_0]),
        I2cTransaction::write(ADDR, vec![0x03, 0x00, 0b0000000_1]),
    ];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);
    d12.set_power_state(PowerState::On)
        .expect("Shouldn't panic on turning dac on");

    d12.set_power_state(PowerState::Down)
        .expect("Shouldn't panic on turning dac off");

    i2c.done();
}

// GAIN Register
#[test]
fn read_reference_divider() {
    let expectations = [
        I2cTransaction::write_read(ADDR, vec![0b0000_0100 as u8], vec![0x00, 0x00]),
        I2cTransaction::write_read(ADDR, vec![0b0000_0100 as u8], vec![0x01, 0x00]),
    ];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    let divider = d12
        .get_reference_divider()
        .expect("Should not panic getting reference divider");
    assert_eq!(divider, ReferenceDivider::None);

    let divider = d12
        .get_reference_divider()
        .expect("Should not panic getting reference divider");
    assert_eq!(divider, ReferenceDivider::Two);

    i2c.done();
}

#[test]
fn set_reference_divider() {
    // NOTE: Default value of BUFF-GAIN bit is 1
    let expectations = [
        I2cTransaction::write(ADDR, vec![0x04, 0b0000000_0, 0x01]),
        I2cTransaction::write(ADDR, vec![0x04, 0b0000000_1, 0x01]),
    ];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);
    d12.set_reference_divider(ReferenceDivider::None)
        .expect("Shouldn't panic on changing reference divider");

    d12.set_reference_divider(ReferenceDivider::Two)
        .expect("Shouldn't panic on changing reference divider");

    i2c.done();
}

#[test]
fn read_buffer_gain() {
    let expectations = [
        I2cTransaction::write_read(ADDR, vec![0b0000_0100 as u8], vec![0x00, 0x00]),
        I2cTransaction::write_read(ADDR, vec![0b0000_0100 as u8], vec![0x00, 0x01]),
    ];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    let gain = d12.get_output_gain().expect("Shouldn't panic reading gain");
    assert_eq!(gain, BufferGain::None);

    let gain = d12.get_output_gain().expect("Shouldn't panic reading gain");
    assert_eq!(gain, BufferGain::Two);

    i2c.done();
}

#[test]
fn set_buffer_gain() {
    // NOTE: Default value of BUFF-GAIN bit is 1
    let expectations = [
        I2cTransaction::write(ADDR, vec![0x04, 0x00, 0b0000000_0]),
        I2cTransaction::write(ADDR, vec![0x04, 0x00, 0b0000000_1]),
    ];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);
    d12.set_output_gain(BufferGain::None)
        .expect("Shouldn't panic on changing buffer gain");

    d12.set_output_gain(BufferGain::Two)
        .expect("Shouldn't panic on changing buffer gain");

    i2c.done();
}

// TRIGGER Register
#[test]
fn set_load_dac() {
    let expectations = [I2cTransaction::write(ADDR, vec![0x05, 0x00, 0b000_1_0000])];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    d12.set_load_dac()
        .expect("Triggering load should not panic");
    i2c.done();
}

#[test]
fn set_soft_reset() {
    let expectations = [I2cTransaction::write(ADDR, vec![0x05, 0x00, 0b000_1010])];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    d12.soft_reset().expect("Soft reset should not panic");
    i2c.done();
}

// STATUS Register
#[test]
fn read_alarm() {
    let expectations = [
        I2cTransaction::write_read(ADDR, vec![0b0000_0111 as u8], vec![0x00, 0x00]),
        I2cTransaction::write_read(ADDR, vec![0b0000_0111 as u8], vec![0x00, 0x01]),
    ];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    let alarm = d12
        .get_alarm_status()
        .expect("Should not panic fetching alarm");
    assert_eq!(alarm, AlarmStatus::Normal);

    let alarm = d12
        .get_alarm_status()
        .expect("Should not panic fetching alarm");
    assert_eq!(alarm, AlarmStatus::Alarm);

    i2c.done();
}

// DAC Register
#[test]
fn set_output_0() {
    let expectations = [I2cTransaction::write(ADDR, vec![0x08, 0x00, 0x00])];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);
    d12.set_output_level(0 as u16)
        .expect("Shouldn't panic on setting dac to 0 output");

    i2c.done();
}

#[test]
fn set_output_max_err() {
    let expectations = [];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    assert_matches!(
        d12.set_output_level(u16::MAX),
        Err(dacx0501::DacError::ValueOverflow)
    );

    i2c.done();
}

#[test]
fn set_output_max() {
    let expectations = [I2cTransaction::write(ADDR, vec![0x08, 0xFF, 0xFF])];
    let mut i2c = I2cMock::new(&expectations);
    let mut d16 = dacx0501::Dac80501::new_i2c(&mut i2c, ADDR);

    assert_matches!(d16.set_output_level(u16::MAX), Ok(()));

    i2c.done();
}

#[test]
fn set_output_mid_scale() {
    let expectations = [I2cTransaction::write(ADDR, vec![0x08, 0x80, 0x00])];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    d12.set_output_level(2048)
        .expect("Setting to mid scale should not panic");

    i2c.done();
}

#[test]
fn read_output_level_12() {
    let expectations = [I2cTransaction::write_read(
        ADDR,
        vec![0b0000_1000 as u8],
        vec![0x80, 0x00],
    )];
    let mut i2c = I2cMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_i2c(&mut i2c, ADDR);

    let level = d12
        .get_output_level()
        .expect("Should not panic fetching output level");
    assert_eq!(level, 2048);

    i2c.done();
}

#[test]
fn read_output_level_16() {
    let expectations = [I2cTransaction::write_read(
        ADDR,
        vec![0b0000_1000 as u8],
        vec![0x80, 0x00],
    )];
    let mut i2c = I2cMock::new(&expectations);
    let mut d16 = dacx0501::Dac80501::new_i2c(&mut i2c, ADDR);

    let level = d16
        .get_output_level()
        .expect("Should not panic fetching output level");
    assert_eq!(level, 32768);

    i2c.done();
}

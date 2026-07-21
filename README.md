
# DACx0501

[![crates.io](https://img.shields.io/crates/d/dacx0501.svg)](https://crates.io/crates/dacx0501)
[![crates.io](https://img.shields.io/crates/v/dacx0501.svg)](https://crates.io/crates/dacx0501)
[![Documentation](https://docs.rs/dacx0501/badge.svg)](https://docs.rs/dacx0501)

This crate is an embedded-hal driver library implementation for the Texas Instruments 80501, 70501 and 60501 DACs. It relies on the embedded-hal ^1.0.0 traits being implemented in the board hal.

## What is supported

The driver supports all write based commands for SPI including an output level set without bounds checking. 
I2C is unsupported but planned.

## What still needs to be implemented

There is no support from the device for reading registers over SPI. Therefore the driver does not support reading the DEVID, SYNC, CONFIG, GAIN, STATUS, or DACDATA registers.

I2C Support.

## Example setting a sine table on one dac and setting a constant value on another

```rust
let mut dac_one = Dac80501::new(spi_one);
let mut dac_two = Dac60501::new(spi_two);

// The dac one output will now be pulled to ground and have no output
dac_one.set_power_state(dacx0501::PowerState::Down).unwrap();

dac_two
    .set_reference_divider(dacx0501::ReferenceDivider::Two)
    .unwrap();
dac_two.set_output_gain(dacx0501::BufferGain::Two).unwrap();


for val in sin_table::SINE_TABLE.iter().cycle() {
    let mut dac_one_val = 4095;
    dac_two.set_output_level(*val).unwrap();
    dac_one.set_output_level(dac_one_val).unwrap();
}
```

Issues and pull requests are welcome
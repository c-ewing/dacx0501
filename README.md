
# DACx0501
 
[![crates.io](https://img.shields.io/crates/d/dacx0501.svg)](https://crates.io/crates/dacx0501)
[![crates.io](https://img.shields.io/crates/v/dacx0501.svg)](https://crates.io/crates/dacx0501)
[![Documentation](https://docs.rs/dacx0501/badge.svg)](https://docs.rs/dacx0501)
 
This crate is an embedded-hal driver library implementation for the Texas Instruments 80501, 70501 and 60501 DACs. It relies on the embedded-hal 1.0.0 (optionally, embedded-hal-async 1.0.0) traits being implemented in the board hal. See the [product page](https://www.ti.com/product/DAC80501/part-details/DAC80501ZDQFT) for the datasheet and other notes.


## Interfaces: SPI and I2C
 
Both SPI and I2C are supported for all write operations (setting output level, power state, gain, reference divider, sync mode, and triggering loads/resets).
 
Register **reads** (resolution, reset value, sync mode, internal reference, power state, reference divider, output gain, alarm status, and output level) are only available over **I2C**. The DAC80501/70501/60501 have no way to send data back to the host over SPI, so those methods simply don't exist on a `Dac` instance constructed with `new_spi`.


## Sync vs async
 
The crate by default is blocking but an async implementation can be enabled with the `async` feature flag. 
 
| Feature | Default | Generates |
|---|---|---|
| `sync` | **on** | `Dac80501`, `Dac70501`, `Dac60501` — blocking, built on `embedded-hal` 1.0 |
| `async` | off | `AsyncDac80501`, `AsyncDac70501`, `AsyncDac60501` — `async`/`.await`, built on `embedded-hal-async` 1.0 |
 
 
If the `sync` version is not required, `default-features = false` can be added alongside the `async` feature:
```toml
[dependencies]
dacx0501 = { version = "0.2", default-features = false, features = ["async"] }
```

## Examples
 
### Blocking, SPI
 
```rust
use dacx0501::{Dac80501, Dac60501, PowerState, ReferenceDivider, BufferGain};
 
let mut dac_one = Dac80501::new_spi(spi_one);
let mut dac_two = Dac60501::new_spi(spi_two);
 
// dac_one's output is now pulled to ground
dac_one.set_power_state(PowerState::Down).unwrap();
 
dac_two.set_reference_divider(ReferenceDivider::Two).unwrap();
dac_two.set_output_gain(BufferGain::Two).unwrap();
 
dac_two.set_output_level(2048).unwrap();
```
 
### Blocking, I2C
 
I2C supports reading back from the device:
 
```rust
use dacx0501::Dac60501;
 
const ADDR: u8 = 0x48;
let mut dac = Dac60501::new_i2c(i2c, ADDR);
 
dac.set_output_level(2048).unwrap();
let level = dac.get_output_level().unwrap();
let resolution = dac.get_resolution().unwrap();
```
 
### Async, SPI (requires the `async` feature)
 
```rust
use dacx0501::AsyncDac70501;
 
let mut dac = AsyncDac70501::new_spi(spi);
dac.set_output_level(8192).await.unwrap();
```
 
### Async, I2C (requires the `async` feature)
 
```rust
use dacx0501::AsyncDac80501;
 
const ADDR: u8 = 0x48;
let mut dac = AsyncDac80501::new_i2c(i2c, ADDR);
 
dac.set_output_level(u16::MAX).await.unwrap();
let status = dac.get_alarm_status().await.unwrap();
```


## Issues and pull requests are welcome
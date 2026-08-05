use dacx0501;
use dacx0501::BufferGain;
use dacx0501::InternalReference;
use dacx0501::UpdateMode;
use dacx0501::PowerState;
use dacx0501::ReferenceDivider;
use dacx0501::Register;
use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};

#[maybe_async_cfg::maybe(
    idents(Dac80501(sync, async = "AsyncDac80501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn construction_16() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let _d16 = dacx0501::Dac80501::new_spi(&mut spi);

    spi.done();
}

#[maybe_async_cfg::maybe(
    idents(Dac70501(sync, async = "AsyncDac70501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn construction_14() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let _d14 = dacx0501::Dac70501::new_spi(&mut spi);

    spi.done();
}

#[maybe_async_cfg::maybe(
    idents(Dac60501(sync, async = "AsyncDac60501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn construction_12() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let _d12 = dacx0501::Dac60501::new_spi(&mut spi);

    spi.done();
}

// NOOP Register
#[maybe_async_cfg::maybe(
    idents(Dac60501(sync, async = "AsyncDac60501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn set_noop() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::NOOP as u8, 0x00, 0x00]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_spi(&mut spi);

    d12.noop()
        .await
        .expect("Writing to noop should not panic");
    spi.done();
}

// Sync Register
#[maybe_async_cfg::maybe(
    idents(Dac60501(sync, async = "AsyncDac60501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn set_sync() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::SYNC as u8, 0x00, 0x00]),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::SYNC as u8, 0x00, 0x01]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_spi(&mut spi);

    d12.set_update_mode(UpdateMode::Asynchronous)
        .await
        .expect("Should not panic setting async");
    d12.set_update_mode(UpdateMode::Synchronous)
        .await
        .expect("Should not panic setting sync");
    spi.done();
}

// Config Register
#[maybe_async_cfg::maybe(
    idents(Dac60501(sync, async = "AsyncDac60501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn set_reference() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::CONFIG as u8, 0b0000000_0, 0x00]),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::CONFIG as u8, 0b0000000_1, 0x00]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_spi(&mut spi);
    d12.set_internal_reference(InternalReference::Enabled)
        .await
        .expect("Shouldn't panic on turning reference on");

    d12.set_internal_reference(InternalReference::Disabled)
        .await
        .expect("Shouldn't panic on turning reference off");

    spi.done();
}

#[maybe_async_cfg::maybe(
    idents(Dac60501(sync, async = "AsyncDac60501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn set_powerdown() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::CONFIG as u8, 0x00, 0b0000000_0]),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::CONFIG as u8, 0x00, 0b0000000_1]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_spi(&mut spi);
    d12.set_power_state(PowerState::On)
        .await
        .expect("Shouldn't panic on turning dac on");

    d12.set_power_state(PowerState::Down)
        .await
        .expect("Shouldn't panic on turning dac off");

    spi.done();
}

// GAIN Register
#[maybe_async_cfg::maybe(
    idents(Dac60501(sync, async = "AsyncDac60501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn set_reference_divider() {
    // NOTE: Default value of BUFF-GAIN bit is 1
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::GAIN as u8, 0b0000000_0, 0x01]),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::GAIN as u8, 0b0000000_1, 0x01]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_spi(&mut spi);
    d12.set_reference_divider(ReferenceDivider::None)
        .await
        .expect("Shouldn't panic on changing reference divider");

    d12.set_reference_divider(ReferenceDivider::Two)
        .await
        .expect("Shouldn't panic on changing reference divider");

    spi.done();
}

#[maybe_async_cfg::maybe(
    idents(Dac60501(sync, async = "AsyncDac60501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn set_buffer_gain() {
    // NOTE: Default value of BUFF-GAIN bit is 1
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::GAIN as u8, 0x00, 0b0000000_0]),
        SpiTransaction::transaction_end(),
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::GAIN as u8, 0x00, 0b0000000_1]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_spi(&mut spi);
    d12.set_output_gain(BufferGain::None)
        .await
        .expect("Shouldn't panic on changing buffer gain");

    d12.set_output_gain(BufferGain::Two)
        .await
        .expect("Shouldn't panic on changing buffer gain");

    spi.done();
}

// TRIGGER Register
#[maybe_async_cfg::maybe(
    idents(Dac60501(sync, async = "AsyncDac60501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn set_load_dac() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::TRIGGER as u8, 0x00, 0b000_1_0000]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_spi(&mut spi);

    d12.load_dac()
        .await
        .expect("Triggering load should not panic");
    spi.done();
}

#[maybe_async_cfg::maybe(
    idents(Dac60501(sync, async = "AsyncDac60501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn set_soft_reset() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::TRIGGER as u8, 0x00, 0b000_1010]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_spi(&mut spi);

    d12.soft_reset().await.expect("Soft reset should not panic");
    spi.done();
}

// DAC Register
#[maybe_async_cfg::maybe(
    idents(Dac60501(sync, async = "AsyncDac60501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn set_output_0() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::DACDATA as u8, 0x00, 0x00]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_spi(&mut spi);
    d12.set_output_level(0 as u16)
        .await
        .expect("Shouldn't panic on setting dac to 0 output");

    spi.done();
}

#[maybe_async_cfg::maybe(
    idents(Dac60501(sync, async = "AsyncDac60501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn set_output_max_err() {
    let expectations = [];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_spi(&mut spi);

    let res = d12.set_output_level(u16::MAX).await;
    assert!(matches!(res, Err(dacx0501::DacError::ValueOverflow)));

    spi.done();
}

#[maybe_async_cfg::maybe(
    idents(Dac80501(sync, async = "AsyncDac80501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn set_output_max() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::DACDATA as u8, 0xFF, 0xFF]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d16 = dacx0501::Dac80501::new_spi(&mut spi);

    let res = d16.set_output_level(u16::MAX).await;
    assert!(matches!(res, Ok(())));

    spi.done();
}

#[maybe_async_cfg::maybe(
    idents(Dac60501(sync, async = "AsyncDac60501")),
    sync(feature = "sync", inner(test)),
    async(feature = "async", inner(tokio::test))
)]
async fn set_output_mid_scale() {
    let expectations = [
        SpiTransaction::transaction_start(),
        SpiTransaction::write_vec(vec![Register::DACDATA as u8, 0x80, 0x00]),
        SpiTransaction::transaction_end(),
    ];
    let mut spi = SpiMock::new(&expectations);
    let mut d12 = dacx0501::Dac60501::new_spi(&mut spi);

    d12.set_output_level(2048)
        .await
        .expect("Setting to mid scale should not panic");

    spi.done();
}

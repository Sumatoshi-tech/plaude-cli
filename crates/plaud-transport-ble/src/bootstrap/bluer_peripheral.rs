//! Real BlueZ GATT peripheral for the `auth bootstrap` flow.
//!
//! Advertises as `PLAUD_NOTE` with the vendor service `0x1910`,
//! captures the phone app's auth write on characteristic `0x2BB1`,
//! and feeds it into a [`BootstrapSession`] which extracts the token
//! and sends back a mock auth-accepted notification on `0x2BB0`.
//!
//! Requires the `btleplug-backend` feature (which pulls in `bluer`).

use std::{collections::BTreeMap, sync::Arc, time::Duration};

use bluer::{
    adv::Advertisement,
    gatt::local::{
        Application, Characteristic, CharacteristicNotify, CharacteristicNotifyMethod, CharacteristicWrite, CharacteristicWriteMethod,
        Service,
    },
};
use bytes::Bytes;
use tokio::sync::{Mutex, mpsc};
use uuid::Uuid;

use super::session::{BootstrapChannel, BootstrapError, BootstrapOutcome, BootstrapSession};
use crate::constants::DEFAULT_CHANNEL_CAPACITY;

/// Plaud vendor service (squatted 16-bit BLE UUID).
const VENDOR_SERVICE_UUID: Uuid = Uuid::from_u128(0x00001910_0000_1000_8000_00805f9b34fb);

/// Vendor write characteristic (phone → device).
const VENDOR_WRITE_UUID: Uuid = Uuid::from_u128(0x00002bb1_0000_1000_8000_00805f9b34fb);

/// Vendor notify characteristic (device → phone).
const VENDOR_NOTIFY_UUID: Uuid = Uuid::from_u128(0x00002bb0_0000_1000_8000_00805f9b34fb);

/// Nordic Semiconductor manufacturer ID from advertising data.
const NORDIC_MANUFACTURER_ID: u16 = 0x0059;

/// Advertised local name.
const LOCAL_NAME: &str = "PLAUD_NOTE";

/// Run a real BlueZ GATT peripheral that captures the phone app's
/// auth token via the bootstrap protocol.
///
/// 1. Powers on the adapter
/// 2. Registers a GATT application with the vendor service
/// 3. Starts advertising as `PLAUD_NOTE`
/// 4. Waits for the phone to connect and write the auth frame
/// 5. Returns the captured token
///
/// # Errors
///
/// Returns [`BootstrapError`] on timeout, decode failure, or BlueZ errors.
pub async fn run_bluer_bootstrap(timeout_budget: Duration) -> Result<BootstrapOutcome, BootstrapError> {
    let session = bluer::Session::new().await.map_err(|e| BootstrapError::DecodeFailed {
        reason: format!("BlueZ session: {e}"),
    })?;
    let adapter = session.default_adapter().await.map_err(|e| BootstrapError::DecodeFailed {
        reason: format!("BlueZ adapter: {e}"),
    })?;
    adapter.set_powered(true).await.map_err(|e| BootstrapError::DecodeFailed {
        reason: format!("power on adapter: {e}"),
    })?;

    // Channel pair: phone writes → peripheral reads
    let (write_tx, write_rx) = mpsc::channel::<Bytes>(DEFAULT_CHANNEL_CAPACITY);
    let (notify_tx, mut notify_rx) = mpsc::channel::<Bytes>(DEFAULT_CHANNEL_CAPACITY);

    let bootstrap_channel = BootstrapChannel {
        writes_in: write_rx,
        notify_out: notify_tx,
    };

    // Shared channel for forwarding notifications from the bootstrap
    // session to the GATT notify characteristic. Wrapped in Arc<Mutex>
    // because bluer's CharacteristicNotifyMethod::Fun requires Fn
    // (not FnOnce).
    let (notify_io_tx, notify_io_rx) = mpsc::channel::<Vec<u8>>(DEFAULT_CHANNEL_CAPACITY);
    let notify_io_rx = Arc::new(Mutex::new(notify_io_rx));

    // Spawn a task that forwards BootstrapSession's notifications to
    // the shared channel.
    tokio::spawn(async move {
        while let Some(data) = notify_rx.recv().await {
            let _ = notify_io_tx.send(data.to_vec()).await;
        }
    });

    // Register the GATT application
    let write_tx_clone = write_tx.clone();
    let app = Application {
        services: vec![Service {
            uuid: VENDOR_SERVICE_UUID,
            primary: true,
            characteristics: vec![
                // Write characteristic (phone → device)
                Characteristic {
                    uuid: VENDOR_WRITE_UUID,
                    write: Some(CharacteristicWrite {
                        write_without_response: true,
                        method: CharacteristicWriteMethod::Fun(Box::new(move |new_value, _req| {
                            let tx = write_tx_clone.clone();
                            Box::pin(async move {
                                let _ = tx.send(Bytes::from(new_value)).await;
                                Ok(())
                            })
                        })),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                // Notify characteristic (device → phone)
                Characteristic {
                    uuid: VENDOR_NOTIFY_UUID,
                    notify: Some(CharacteristicNotify {
                        notify: true,
                        method: CharacteristicNotifyMethod::Fun(Box::new({
                            let rx = Arc::clone(&notify_io_rx);
                            move |mut notifier| {
                                let rx = Arc::clone(&rx);
                                Box::pin(async move {
                                    let mut guard = rx.lock().await;
                                    while let Some(data) = guard.recv().await {
                                        if notifier.notify(data).await.is_err() {
                                            break;
                                        }
                                    }
                                })
                            }
                        })),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }],
        ..Default::default()
    };

    let _app_handle = adapter
        .serve_gatt_application(app)
        .await
        .map_err(|e| BootstrapError::DecodeFailed {
            reason: format!("GATT application: {e}"),
        })?;

    // Advertise as PLAUD_NOTE
    let adv = Advertisement {
        local_name: Some(LOCAL_NAME.to_owned()),
        advertisement_type: bluer::adv::Type::Peripheral,
        service_uuids: vec![VENDOR_SERVICE_UUID].into_iter().collect(),
        manufacturer_data: BTreeMap::from([(NORDIC_MANUFACTURER_ID, vec![])]),
        ..Default::default()
    };

    let _adv_handle = adapter.advertise(adv).await.map_err(|e| BootstrapError::DecodeFailed {
        reason: format!("advertisement: {e}"),
    })?;

    eprintln!("Advertising as {LOCAL_NAME} — open the Plaud app and pair with this device...");

    // Run the bootstrap protocol
    let bootstrap = BootstrapSession::new(bootstrap_channel);
    bootstrap.run(timeout_budget).await
}

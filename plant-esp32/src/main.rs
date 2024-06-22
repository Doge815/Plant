use std::sync::Arc;

use esp32_gpio_wrapper::GpioWrapper;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::{esp, EspError};
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, ClientConfiguration, Configuration, EspWifi};
use log::info;
use plant_db::PlantDB;
use tokio::sync::Mutex;

mod plant;
mod server;
mod plant_db;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/wifi.rs"));

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Setting up eventfd...");
    let config = esp_idf_svc::sys::esp_vfs_eventfd_config_t {
        max_fds: 5,
        ..Default::default()
    };
    esp! { unsafe { esp_idf_svc::sys::esp_vfs_eventfd_register(&config) } }?;

    info!("Setting up board....");
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let timer = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    info!("Initializing Wi-Fi...");
    let wifi = AsyncWifi::wrap(
        EspWifi::new(peripherals.modem, sysloop.clone(), Some(nvs.clone()))?,
        sysloop,
        timer.clone(),
    )?;

    info!("Starting async run loop");
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async move {
            let mut wifi_loop = WifiLoop { wifi };
            wifi_loop.configure().await?;
            wifi_loop.initial_connect().await?;

            let gpio = GpioWrapper::new(Some(peripherals.adc1), None, peripherals.pins);
            let plants = Arc::new(Mutex::new(PlantDB::new(nvs)));
            tokio::spawn(plant::measure_plants( gpio.clone(), plants.clone()));
            tokio::spawn(server::auxum_serve(plants.clone()));

            info!("Entering main Wi-Fi run loop...");
            wifi_loop.stay_connected().await
        })?;

    Ok(())
}

pub struct WifiLoop<'a> {
    wifi: AsyncWifi<EspWifi<'a>>,
}

impl<'a> WifiLoop<'a> {
    pub async fn configure(&mut self) -> Result<(), EspError> {
        info!("Setting Wi-Fi credentials...");
        info!("Wifi SSID: {}", WIFI_SSID);
        info!("Wifi PSK: {}", WIFI_PSK);

        self.wifi
            .set_configuration(&Configuration::Client(ClientConfiguration {
                ssid: WIFI_SSID.parse().unwrap(),
                password: WIFI_PSK.parse().unwrap(),
                ..Default::default()
            }))?;

        info!("Starting Wi-Fi driver...");
        self.wifi.start().await
    }

    pub async fn initial_connect(&mut self) -> Result<(), EspError> {
        self.do_connect_loop(true).await
    }

    pub async fn stay_connected(mut self) -> Result<(), EspError> {
        self.do_connect_loop(false).await
    }

    async fn do_connect_loop(&mut self, exit_after_first_connect: bool) -> Result<(), EspError> {
        let wifi = &mut self.wifi;
        loop {
            wifi.wifi_wait(|wifi| wifi.is_up(), None).await?;
            info!("Connecting to Wi-Fi...");
            wifi.connect().await?;
            info!("Waiting for association...");
            wifi.ip_wait_while(|wifi| wifi.is_up().map(|s| !s), None)
                .await?;
            if exit_after_first_connect {
                return Ok(());
            }
        }
    }
}

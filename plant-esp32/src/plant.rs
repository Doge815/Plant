use std::sync::Arc;

use esp32_gpio_wrapper::{GpioWrapper, MeasurementConfig};
use log::warn;
use plant_common::{Connector, Moisture, PlantInfo};
use ringbuffer::{AllocRingBuffer, RingBuffer};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::plant_db;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PlantData {
    pub id: u16,
    pub connection: Connector,
    pub name: String,
}

#[derive(Clone)]
pub struct Plant {
    pub info: PlantData,
    pub measured_values: AllocRingBuffer<f32>,
}

impl From<PlantData> for Plant {
    fn from(plant: PlantData) -> Self {
        Plant {
            info: plant,
            measured_values: AllocRingBuffer::new(60),
        }
    }
}

impl From<Plant> for PlantInfo {
    fn from(plant: Plant) -> Self {
        let moisture = if plant.measured_values.len() > 0 {
            Some(plant.measured_values.iter().sum::<f32>() / plant.measured_values.len() as f32)
        } else {
            None
        };
        PlantInfo {
            id: plant.info.id,
            name: plant.info.name,
            measured_moisture: Moisture {
                measured_voltage: moisture,
                soil: plant_common::SoilType::PottingSoil,
                pot_volume: None,
            },
            connection: plant.info.connection,
        }
    }
}


pub async fn measure_plants(gpio: GpioWrapper, plants: Arc<Mutex<plant_db::PlantDB>>) {
    const MEASUREMENT_CONFIG: MeasurementConfig = MeasurementConfig {
        to_measure: 32,
        attenuation: esp32_gpio_wrapper::Attenuation::DB11,
    };
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let mut data = plants.lock().await;
        for plant in data.plants_iter_mut() {
            let value = match plant.info.connection {
                Connector::GPIO(pin) => gpio
                    .get_pin(pin as usize)
                    .unwrap()
                    .get_adc_averaged(MEASUREMENT_CONFIG)
                    .await
                    .unwrap(),
            };
            plant.measured_values.push(value);
        }
    }
}

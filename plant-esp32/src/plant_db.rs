use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault};
use log::{error, info};
use plant_common::Connector;

use crate::plant::{Plant, PlantData};

pub struct PlantDB {
    board_name: String,
    plants: Vec<Plant>,
    nvs: EspNvs<NvsDefault>,
    next_id: u16,
}
const NAMESPACE: &str = "plant_ns";
impl PlantDB {
    pub fn new(nvs: EspNvsPartition<NvsDefault>) -> PlantDB {
        let nvs = EspNvs::new(nvs, NAMESPACE, true).expect("Could't get namespace");

        let next_id: u16 = match nvs.get_u16("next_id") {
            Ok(plant_count) => match plant_count {
                Some(v) => {
                    info!("Read 'next_id' as {}", v);
                    v
                }
                None => {
                    error!("Read 'next_id' as 'None' from NVS.");
                    nvs.set_u16("next_id", 0).unwrap();
                    0
                }
            },
            Err(e) => {
                error!("Cannot read 'next id' from NVS: {:?}.", e);
                nvs.set_u16("next_id", 0).unwrap();
                0
            }
        };
        let plants: u16 = match nvs.get_u16("plant_count") {
            Ok(plant_count) => match plant_count {
                Some(v) => {
                    info!("Read plant count as '{}'", v);
                    v
                }
                None => {
                    error!("Read plant count as 'None' from NVS.");
                    nvs.set_u16("plant_count", 0).unwrap();
                    0
                }
            },
            Err(e) => {
                error!("Cannot read plant count from NVS: {:?}.", e);
                nvs.set_u16("plant_count", 0).unwrap();
                0
            }
        };
        let mut plant_infos = vec![];
        for i in 0..plants {
            let key = format!("plant_{}", i);
            let mut buf: &mut [u8] = &mut [0; 100];
            match nvs.get_raw(&key, &mut buf) {
                Ok(v) => {
                    if let Some(obj) = v {
                        let plant: PlantData = postcard::from_bytes::<PlantData>(obj).unwrap();
                        info!("Read plant {} as '{:?}'", i, plant);
                        plant_infos.push(plant);
                    }
                }
                Err(e) => {
                    error!("Cannot read plant {} from NVS: {:?}.", i, e);
                }
            }
        }
        PlantDB {
            board_name: "Board1".to_string(),
            plants: plant_infos.into_iter().map(|x| x.into()).collect(),
            nvs,
            next_id,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.board_name
    }

    pub fn get_plants(&self) -> &Vec<Plant> {
        &self.plants
    }
    pub fn plants_iter_mut(&mut self) -> std::slice::IterMut<Plant> {
        self.plants.iter_mut()
    }

    pub fn create_plant(&mut self, name: String, connection: Connector) -> Result<(), ()> {
        //TODO: check if connection is used

        let plant = PlantData {
            id: self.next_id,
            name,
            connection,
        };
        self.next_id += 1;
        self.plants.push(plant.into());
        self.nvs.set_u16("next_id", self.next_id).unwrap();
        self.nvs
            .set_u16("plant_count", self.plants.len() as u16)
            .unwrap();
        let key = format!("plant_{}", self.plants.len() - 1);
        let buf: &mut [u8] = &mut [0; 100];
        let buf = postcard::to_slice(&self.plants[self.plants.len() - 1].info, buf).unwrap();
        self.nvs.set_raw(&key, &buf).unwrap();
        Ok(())
    }

    #[must_use]
    fn get_index(&self, id: u16) -> Option<usize> {
        //maybe add a hashset to the struct
        for (i, plant) in self.plants.iter().enumerate() {
            if plant.info.id == id {
                return Some(i);
            }
        }
        None
    }

    pub fn update_plant(&mut self, id: u16, name: String, connection: Connector) -> Result<(), ()> {
        if let Some(index) = self.get_index(id) {
            self.plants[index].info.name = name;
            self.plants[index].info.connection = connection;
            let key = format!("plant_{}", id);
            let buf: &mut [u8] = &mut [0; 100];
            let buf = postcard::to_slice(&self.plants[index].info, buf).unwrap();
            self.nvs.set_raw(&key, &buf).unwrap();
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn delete_plant(&mut self, id: u16) -> Result<(), ()> {
        if let Some(index) = self.get_index(id) {
            //deleted plant is last plant
            if index + 1 == self.plants.len() {
                self.plants.pop();
            } else {
                //remove the deleted plant and store the last plant at the index of the deleted plant
                self.plants.swap_remove(index);
                let key = format!("plant_{}", id);
                let buf: &mut [u8] = &mut [0; 100];
                let buf = postcard::to_slice(&self.plants[index].info, buf).unwrap();
                self.nvs.set_raw(&key, &buf).unwrap();
            }
            self.nvs
                .set_u16("plant_count", self.plants.len() as u16)
                .unwrap();
            Result::Ok(())
        } else {
            Err(())
        }
    }
}

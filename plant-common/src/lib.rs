#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize, Default)]
pub enum SoilType {
    #[default]
    PottingSoil,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub enum CalculatedMoisture {
    Unknown,
    VeryDry,
    Dry,
    Perfect,
    Moist,
    Wet,
}

impl std::fmt::Display for CalculatedMoisture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalculatedMoisture::Unknown => write!(f, "Unknown"),
            CalculatedMoisture::VeryDry => write!(f, "Very dry"),
            CalculatedMoisture::Dry => write!(f, "Dry"),
            CalculatedMoisture::Perfect => write!(f, "Perfect"),
            CalculatedMoisture::Moist => write!(f, "Moist"),
            CalculatedMoisture::Wet => write!(f, "Wet"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize, Default)]
pub struct Moisture {
    pub measured_voltage: Option<f32>,
    pub pot_volume: Option<f32>,
    pub soil: SoilType,
}

impl Moisture {
    #[must_use]
    pub fn calulated_moisture(&self) -> CalculatedMoisture {
        match self.measured_voltage {
            None => return CalculatedMoisture::Unknown,
            Some(volatage) => {
                if volatage < 500.0 {
                    return CalculatedMoisture::Wet;
                } else if volatage < 1000.0 {
                    return CalculatedMoisture::Moist;
                } else if volatage < 1500.0 {
                    return CalculatedMoisture::Perfect;
                } else if volatage < 2500.0 {
                    return CalculatedMoisture::Dry;
                } else {
                    return CalculatedMoisture::VeryDry;
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Debug)]
pub enum Connector {
    GPIO(u8),
    //I2C
}
impl Default for Connector {
    fn default() -> Self {
        Connector::GPIO(32)
    }
    
}


#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize, Default)]
pub struct PlantInfo {
    pub id: u16,
    pub name: String,
    pub measured_moisture: Moisture,
    pub connection: Connector,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct BoardState {
    pub name: String,
    pub plants: Vec<PlantInfo>,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum OkStatus {
    Empty,
    Created,
    Deleted,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ErrStatus {
    BadRequest,
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ReplyStatus {
    Ok(OkStatus),
    Err(ErrStatus),
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Reply {
    pub status: ReplyStatus,
    pub state: BoardState,
}

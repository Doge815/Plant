use std::{clone, net::Ipv4Addr};

use plant_common::{BoardState, Connector, Moisture, Reply};
use reqwest::RequestBuilder;
use tokio_with_wasm::tokio::sync::mpsc::Sender;

use crate::app::BoardReply;

/*impl std::fmt::Display for IpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.ip[0], self.ip[1], self.ip[2], self.ip[3]
        )
    }
}*/

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub enum OnlineStatus {
    Online,
    LoadingWasOnline,
    LoadingWasOffline,
    Offline,
}

impl std::fmt::Display for OnlineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OnlineStatus::Online => write!(f, "Online"),
            OnlineStatus::LoadingWasOnline => write!(f, "Loading"),
            OnlineStatus::LoadingWasOffline => write!(f, "Loading"),
            OnlineStatus::Offline => write!(f, "Offline"),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Board {
    pub ip: Ipv4Addr,
    pub status: OnlineStatus,
    pub state: Option<BoardState>,
    pub settings_new_plant_name: String,
    pub settings_new_plant_port: u8,
}

impl Board {
    async fn send_request(
        request_builder: RequestBuilder,
        tx: Sender<BoardReply>,
        board: Board,
        on_error: BoardReply,
    ) {
        const MAX_WAIT_TIME: std::time::Duration =
            tokio_with_wasm::tokio::time::Duration::from_secs(5);
        let request = request_builder.send();

        tokio_with_wasm::tokio::select! {
            output = request => {
                if let Ok(response) = output {
                    if let Ok(data) = response.text().await {
                        if let Ok(reply) = serde_json::from_str::<Reply>(&data) {
                            tx.send(BoardReply {
                                message: None,
                                board: Some(Board {
                                    status: OnlineStatus::Online,
                                    state: Some(reply.state),
                                    ..board
                                }),
                            })
                            .await
                            .unwrap();
                            return;
                        }
                    }
                }
            },
            _ = tokio_with_wasm::tokio::time::sleep(MAX_WAIT_TIME) => { },
        };

        tx.send(on_error).await.unwrap();
    }

    fn set_loading(&mut self) {
        if self.status == OnlineStatus::Online || self.status == OnlineStatus::LoadingWasOnline {
            self.status = OnlineStatus::LoadingWasOnline;
        } else {
            self.status = OnlineStatus::LoadingWasOffline;
        }
    }

    pub fn reload(&mut self, tx: Sender<BoardReply>, http_client: reqwest::Client) {
        self.set_loading();
        let clone = self.clone();
        tokio_with_wasm::tokio::spawn(async move {
            let request_builder = http_client.get(format!("http://{}/state", clone.ip.clone()));
            Board::send_request(
                request_builder,
                tx,
                clone.clone(),
                BoardReply {
                    message: None,
                    board: Some(Board {
                        state: None,
                        ..clone.clone()
                    }),
                },
            )
            .await;
        });
    }

    pub fn create_plant(
        &mut self,
        tx: Sender<BoardReply>,
        http_client: reqwest::Client,
        name: String,
        connection: Connector,
    ) {
        self.set_loading();
        let clone = self.clone();
        tokio_with_wasm::tokio::spawn(async move {
            let request_builder = http_client
                .post(format!("http://{}/create_plant", clone.ip))
                .json(&plant_common::PlantInfo {
                    name,
                    connection,
                    ..Default::default()
                });
            Board::send_request(
                request_builder,
                tx,
                clone.clone(),
                BoardReply {
                    message: None,
                    board: Some(Board {
                        state: None,
                        ..clone.clone()
                    }),
                },
            )
            .await;
        });
    }

    pub fn delete_plant(&mut self, tx: Sender<BoardReply>, http_client: reqwest::Client, id: u16) {
        self.set_loading();
        let clone = self.clone();
        tokio_with_wasm::tokio::spawn(async move {
            let request_builder = http_client
                .delete(format!("http://{}/delete_plant", clone.ip))
                .json(&plant_common::PlantInfo {
                    id,
                    ..Default::default()
                });
            Board::send_request(
                request_builder,
                tx,
                clone.clone(),
                BoardReply {
                    message: None,
                    board: Some(Board {
                        state: None,
                        ..clone.clone()
                    }),
                },
            )
            .await;
        });
    }
}

#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Boards {
    pub boards: Vec<Board>,
}
/*
   pub fn create_plant(
       &self,
       tx: Sender<BoardReply>,
       http_client: reqwest::Client,
       name: String,
       connection: Connector,
   ) {
       let clone = self.clone();
       const MAX_WAIT_TIME: std::time::Duration =
           tokio_with_wasm::tokio::time::Duration::from_secs(5);

       tokio_with_wasm::tokio::spawn(async move {
           let request = http_client
               .post(format!("http://{}/create_plant", clone.ip))
               .json(&plant_common::PlantInfo {
                   name,
                   connection,
                   measured_moisture: Moisture {
                       measured_voltage: None,
                       pot_volume: None,
                       soil: plant_common::SoilType::PottingSoil,
                   },
                   id: 0,
               })
               .send();

           tokio_with_wasm::tokio::select! {
               output = request => {
                   if let Ok(response) = output {
                       if let Ok(data) = response.text().await {
                           if let Ok(reply) = serde_json::from_str::<Reply>(&data) {
                               tx.send(BoardReply {
                                   message: None,
                                   board: Some(Board {
                                   status: OnlineStatus::Online,
                                   state: Some(reply.state),
                                   ..clone
                               })
                               })
                               .await
                               .unwrap();
                               return;
                           //todo: handle status
                           }
                       }
                   }
               },
               _ = tokio_with_wasm::tokio::time::sleep(MAX_WAIT_TIME) => { },
           };

           tx.send(BoardReply {
               message: None,
               board: None,
           })
           .await
           .unwrap();
       });
   }
*/

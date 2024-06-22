use std::{net::SocketAddr, sync::Arc};

use axum::{routing::*, Json, Router};
use log::*;
use plant_common::{BoardState, OkStatus, PlantInfo, Reply, ReplyStatus};
use tokio::sync::Mutex;

use crate::plant_db::PlantDB;

pub async fn auxum_serve(plants: Arc<Mutex<PlantDB>>) {
    let app = Router::new()
        .route(
            "/state",
            get({
                let plants = Arc::clone(&plants);
                move || get_current_state(plants)
            }),
        )
        .route(
            "/create_plant",
            post({
                let plants = Arc::clone(&plants);
                move |body| create_plant(plants, body)
            }),
        )
        .route(
            "/delete_plant",
            delete({
                let plants = Arc::clone(&plants);
                move |body| delete_plant(plants, body)
            }),
        )
        ;

    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_current_state(plants: Arc<Mutex<PlantDB>>) -> Json<Reply> {
    let mut db = plants.lock().await;
    let state = BoardState {
        name: db.get_name().clone(),
        plants: db.plants_iter_mut().map(|x| x.clone().into()).collect(),
    };
    drop(db);
    Json(Reply {
        status: ReplyStatus::Ok(OkStatus::Empty),
        state,
    })
}

async fn create_plant(plants: Arc<Mutex<PlantDB>>, request: Json<PlantInfo>) -> Json<Reply> {
    let request = request.0;
    let mut db = plants.lock().await;
    let created = db.create_plant(request.name, request.connection);
    let state = BoardState {
        name: db.get_name().clone(),
        plants: db.plants_iter_mut().map(|x| x.clone().into()).collect(),
    };
    drop(db);
    return match created {
        Ok(_) => Json(Reply {
            status: ReplyStatus::Ok(OkStatus::Created),
            state,
        }),
        //todo: error type
        Err(_) => Json(Reply {
            status: ReplyStatus::Err(plant_common::ErrStatus::BadRequest),
            state,
        }),
    };
}

async fn delete_plant(plants: Arc<Mutex<PlantDB>>, request: Json<PlantInfo>) -> Json<Reply> {
    let request = request.0;
    let mut db = plants.lock().await;
    let delteted = db.delete_plant(request.id);
    let state = BoardState {
        name: db.get_name().clone(),
        plants: db.plants_iter_mut().map(|x| x.clone().into()).collect(),
    };
    drop(db);
    return match delteted {
        Ok(_) => Json(Reply {
            status: ReplyStatus::Ok(OkStatus::Deleted),
            state,
        }),
        //todo: error type
        Err(_) => Json(Reply {
            status: ReplyStatus::Err(plant_common::ErrStatus::BadRequest),
            state,
        }),
    };
}

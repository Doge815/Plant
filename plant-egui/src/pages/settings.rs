use std::net::Ipv4Addr;

use crate::{
    app::App,
    board::{self, OnlineStatus},
};
use egui::Ui;

pub struct SettingsPage {
    pub new_board: Ipv4Addr,
}

impl Default for SettingsPage {
    fn default() -> Self {
        Self {
            new_board: Ipv4Addr::new(192, 168, 178, 198),
        }
    }
}

pub fn settings_page(ui: &mut Ui, app: &mut App) {
    let mut i: i32 = 0;
    while (i as usize) < app.boards.boards.len() {
        let board = app.boards.boards.get_mut(i as usize).unwrap();
        let mut delete = false;
        ui.horizontal(|ui| {
            if let Some(board_state) = &board.state {
                ui.heading(&board_state.name);
                ui.label(board.ip.to_string());
            } else {
                ui.heading(board.ip.to_string());
            }
            match board.status {
                crate::board::OnlineStatus::Online => {
                    ui.colored_label(egui::Color32::GREEN, "Online");
                }
                crate::board::OnlineStatus::LoadingWasOnline => {
                    ui.colored_label(egui::Color32::YELLOW, "Loading");
                }
                crate::board::OnlineStatus::LoadingWasOffline => {
                    ui.colored_label(egui::Color32::YELLOW, "Loading");
                }
                crate::board::OnlineStatus::Offline => {
                    ui.colored_label(egui::Color32::RED, "Offline");
                }
            }
            if ui.button('\u{1F5D1}'.to_string()).clicked() {
                delete = true;
            }
            if ui.button("Refresh").clicked() {
                board.reload(app.board_sender.clone(), app.http_client.clone());
            }
        });
        let board = app.boards.boards.get_mut(i as usize).unwrap();
        if let Some(board_state) = board.state.clone() {
            ui.indent("state", |ui| {
                for plant in &board_state.plants {
                    ui.horizontal(|ui| {
                        ui.strong(format!("{}:", &plant.name));
                        ui.label(format!("{:?}", &plant.connection));
                        if ui
                            .add_enabled(
                                board.status == OnlineStatus::Online,
                                egui::Button::new('\u{1F5D1}'.to_string()),
                            )
                            .clicked()
                        {
                            board.delete_plant(
                                app.board_sender.clone(),
                                app.http_client.clone(),
                                plant.id,
                            );
                        }
                    });
                }
            });
        }
        if board.status == OnlineStatus::Online || board.status == OnlineStatus::LoadingWasOnline {
            ui.horizontal(|ui| {
                ui.label("New plant name:");
                ui.text_edit_singleline(&mut board.settings_new_plant_name);
                ui.label("Port:");
                ui.add(
                    egui::DragValue::new(&mut board.settings_new_plant_port).clamp_range(0..=40),
                );
                if ui
                    .add_enabled(board.status == OnlineStatus::Online, egui::Button::new('\u{2795}'.to_string()))
                    .clicked()
                {
                    board.create_plant(
                        app.board_sender.clone(),
                        app.http_client.clone(),
                        board.settings_new_plant_name.clone(),
                        plant_common::Connector::GPIO(board.settings_new_plant_port),
                    );
                }
            });
        }
        ui.separator();
        if delete {
            app.boards.boards.remove(i as usize);
        } else {
            i += 1;
        }
    }
    ui.heading("Add new board");
    ui.horizontal(|ui| {
        ui.label("IP:");
        let mut octets = app.settings_page.new_board.octets();
        ui.add(egui::DragValue::new(&mut octets[0]).clamp_range(0..=255));
        ui.label(".");
        ui.add(egui::DragValue::new(&mut octets[1]).clamp_range(0..=255));
        ui.label(".");
        ui.add(egui::DragValue::new(&mut octets[2]).clamp_range(0..=255));
        ui.label(".");
        ui.add(egui::DragValue::new(&mut octets[3]).clamp_range(0..=255));
        app.settings_page.new_board = Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]);
        if ui.button('\u{2795}'.to_string()).clicked() {
            app.boards.boards.push(crate::board::Board {
                ip: app.settings_page.new_board,
                status: crate::board::OnlineStatus::Offline,
                state: None,
                settings_new_plant_name: "New plant".to_string(),
                settings_new_plant_port: 0,
            });
            app.boards
                .boards
                .last_mut()
                .unwrap()
                .reload(app.board_sender.clone(), app.http_client.clone());
        }
    });
}

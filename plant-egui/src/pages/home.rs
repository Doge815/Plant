use crate::app::App;
use egui::Ui;

pub fn home_page(ui: &mut Ui, app: &mut App) {
    for i in 0..app.boards.boards.len() {
        let board = &mut app.boards.boards[i];
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
            if ui.button("Refresh").clicked() {
                board.reload(app.board_sender.clone(), app.http_client.clone());
            }
        });
        if let Some(board_state) = &mut board.state {
            ui.indent("state", |ui| {
                for plant in &mut board_state.plants {
                    ui.horizontal(|ui| {
                        ui.strong(format!("{}:", &plant.name));
                        ui.label(plant.measured_moisture.calulated_moisture().to_string());
                    });
                }
            });
        }
        if (i + 1) < app.boards.boards.len() {
            ui.separator();
        }
    }
}

use crate::board::{self, Board, Boards, OnlineStatus};
use std::net::Ipv4Addr;
use tokio_with_wasm::tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone, Default)]
pub struct Message {}

#[derive(Debug, Clone, Default)]
pub struct BoardReply {
    pub board: Option<Board>,
    pub message: Option<Message>,
}

pub enum Page {
    Home,
    Settings,
    Watering,
}

pub struct App {
    pub page: Page,
    pub boards: Boards,
    pub board_sender: Sender<BoardReply>,
    pub board_receiver: Receiver<BoardReply>,
    pub http_client: reqwest::Client,
    pub settings_page: crate::pages::settings::SettingsPage,
}

impl Default for App {
    fn default() -> Self {
        let (board_sender, board_receiver) = tokio_with_wasm::tokio::sync::mpsc::channel(100);
        Self {
            page: Page::Home,
            boards: Boards::default(),
            board_sender,
            board_receiver,
            http_client: reqwest::Client::new(),
            settings_page: crate::pages::settings::SettingsPage::default(),
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app: App = Default::default();
        if let Some(storage) = cc.storage {
            app.boards = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        };
        if !app
            .boards
            .boards
            .iter()
            .map(|x| x.ip)
            .any(|x| x == Ipv4Addr::new(10, 69, 69, 155))
        {
            app.boards.boards.push(Board {
                ip: Ipv4Addr::new(10, 69, 69, 155),
                status: OnlineStatus::Offline,
                state: None,
                settings_new_plant_name: "New Plant".to_string(),
                settings_new_plant_port: 0,
            });
        }
        for board in app.boards.boards.iter_mut() {
            board.reload(app.board_sender.clone(), app.http_client.clone());
        }
        app
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.boards);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Some(board) = self.board_receiver.try_recv().ok() {
            if let Some(mut board) = board.board {
                if let Some(index) = self.boards.boards.iter().position(|b| b.ip == board.ip) {
                    if board.state.is_some() {
                        let mut state = board.state.unwrap();
                        state.plants.sort_by(|x, y| x.id.cmp(&y.id));
                        board.state = Some(state);
                        self.boards.boards[index] = board;
                    } else {
                        self.boards.boards[index].status = OnlineStatus::Offline;
                    }
                }
                //if the board is not in boards, dont add it
                //every created board is added to boards at the time of creation
                //if it is not in boards, it was deleted
            }
            // Todo: handle messages
        }
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Boards").clicked() {
                    self.page = Page::Home;
                }
                if ui.button("Settings").clicked() {
                    self.page = Page::Settings;
                }
                if ui.button("Watering").clicked() {
                    self.page = Page::Watering;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    egui::widgets::global_dark_light_mode_buttons(ui);
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            (match self.page {
                Page::Home => crate::pages::home::home_page,
                Page::Settings => crate::pages::settings::settings_page,
                Page::Watering => crate::pages::home::home_page,
            } as fn(_, _))(ui, self);
        });
    }
}

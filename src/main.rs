mod client;
mod message;
mod model;
mod ui;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4 as gtk;

use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, glib};
use vaux_mqtt::{FixedHeader, Packet};

use crate::client::ClientSetting;
use crate::model::PacketObject;

fn main() -> glib::ExitCode {
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(32);
    let (packet_tx, mut packet_rx) = tokio::sync::mpsc::channel(32);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let _packet_tx = packet_tx.clone();

    std::thread::spawn(move || {
        rt.block_on(async {
            client::run(_packet_tx, cmd_rx).await;
        });
    });

    let app = Application::builder()
        .application_id("org.bytetrail-rs.vaux")
        .build();

    let list_model = gtk::gio::ListStore::new::<PacketObject>();
    let message_model = Rc::new(RefCell::new(list_model));
    let _message_model = Rc::clone(&message_model);
    glib::spawn_future_local(async move {
        while let Some(packet) = packet_rx.recv().await {
            let packet_obj = PacketObject::new(packet);
            (*_message_model.borrow_mut()).append(&packet_obj);
        }
    });

    app.connect_activate(move |app| {
        let client_setting = ClientSetting::new();

        // We create the main window.
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(650)
            .default_height(400)
            .title("Vaux Client")
            .build();

        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        main_box.set_margin_start(10);
        main_box.set_margin_end(10);
        main_box.set_margin_top(10);
        main_box.set_margin_bottom(10);
        main_box.set_spacing(10);

        let session_frame = gtk::Frame::new(Some("Session"));
        main_box.append(&session_frame);
        let sess_grid = gtk::Grid::new();
        sess_grid.set_row_spacing(4);
        sess_grid.set_column_spacing(4);
        sess_grid.set_margin_start(10);
        sess_grid.set_margin_end(10);
        sess_grid.set_margin_top(10);
        sess_grid.set_margin_bottom(10);
        sess_grid.set_hexpand(true);
        let mut row = 0;

        let client_setting_frame = ui::build_settings(&client_setting);
        sess_grid.attach(&client_setting_frame, 0, row, 1, 1);
        let will_frame = ui::build_will(&client_setting);
        sess_grid.attach(&will_frame, 1, row, 1, 1);
        row += 1;
        let tls_frame = ui::build_tls(
            Rc::clone(&client_setting.with_tls),
            Rc::clone(&client_setting.ca_file),
            Rc::clone(&client_setting.client_cert),
        );
        sess_grid.attach(&tls_frame, 0, row, 1, 1);
        let credentials_frame = ui::build_credentials(
            Rc::clone(&client_setting.with_credentials),
            Rc::clone(&client_setting.username),
            Rc::clone(&client_setting.password),
        );
        sess_grid.attach(&credentials_frame, 1, row, 1, 1);
        session_frame.set_child(Some(&sess_grid));
        row += 1;

        let connect_frame = ui::build_actions(cmd_tx.clone(), &client_setting);
        main_box.append(&connect_frame);

        let message_frame = message::build_message_view(Rc::clone(&message_model));
        main_box.append(&message_frame);

        let packet = PacketObject::new(Packet::PingResponse(FixedHeader::new(
            vaux_mqtt::PacketType::PingResp,
        )));
        (*message_model.borrow_mut()).append(&packet);

        window.set_child(Some(&main_box));
        window.present();
    });

    app.run()
}

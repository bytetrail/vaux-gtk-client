use std::{cell::RefCell, rc::Rc, time::Duration};

use glib::clone;
use gtk4::{self as gtk};

use gtk::prelude::*;
use vaux_mqtt::WillMessage;

use crate::client::{self, ClientSetting, Command};

const FRAME_MARGIN: i32 = 6;
const TOPIC_ENTRY_WIDTH_CHARS: i32 = 80;
const PUBLISH_TEXT_WIDTH_REQUEST: i32 = 300;
const PUBLISH_TEXT_HEIGHT_REQUEST: i32 = 120;

pub fn build_actions(
    clean_start_check: &gtk::CheckButton,
    cmd_tx: tokio::sync::mpsc::Sender<Command>,
    client_settings: &ClientSetting,
) -> gtk::Frame {
    let frame = gtk::Frame::new(Some("Actions"));
    let grid = gtk::Grid::new();
    grid.set_column_spacing(4);
    grid.set_row_spacing(4);
    grid.set_margin_bottom(FRAME_MARGIN);
    grid.set_margin_start(FRAME_MARGIN);
    grid.set_margin_end(FRAME_MARGIN);
    grid.set_vexpand(false);
    grid.set_hexpand(true);

    let row = 0;

    let ping_button = gtk::Button::with_label("Ping");
    // Make Ping button same width as Connect/Disconnect button
    ping_button.set_sensitive(false); // Initially disabled
    let conn_button = build_connect(
        &ping_button,
        clean_start_check,
        client_settings,
        cmd_tx.clone(),
    );
    grid.attach(&conn_button, 0, row, 1, 1);

    grid.attach(&ping_button, 0, 1, 1, 1);
    let _cmd_tx = cmd_tx.clone();
    ping_button.connect_clicked(move |_| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        match rt.block_on(async { _cmd_tx.send(client::Command::Ping).await }) {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to send ping command: {e}");
            }
        }
    });

    // attach the subscribe frame to column 1, row, 0, 3 rows height
    let notebook = build_action_notebook(cmd_tx);
    grid.attach(&notebook, 1, 0, 1, 3);

    frame.set_child(Some(&grid));
    frame
}

pub fn build_action_notebook(cmd_tx: tokio::sync::mpsc::Sender<Command>) -> gtk::Notebook {
    let notebook = gtk::Notebook::new();
    notebook.set_tab_pos(gtk::PositionType::Top);
    notebook.set_hexpand(true);
    notebook.set_vexpand(false);

    let pub_frame = build_publish(cmd_tx.clone());
    notebook.append_page(&pub_frame, Some(&gtk::Label::new(Some("Publish"))));
    let sub_frame = build_subscribe(cmd_tx.clone());
    notebook.append_page(&sub_frame, Some(&gtk::Label::new(Some("Subscribe"))));
    let unsub_frame = build_unsubscribe(cmd_tx);
    notebook.append_page(&unsub_frame, Some(&gtk::Label::new(Some("Unsubscribe"))));

    notebook
}

pub(crate) fn build_subscribe(cmd_tx: tokio::sync::mpsc::Sender<Command>) -> gtk::Frame {
    let frame = gtk::Frame::new(Some("Subscribe"));
    let grid = gtk::Grid::new();
    grid.set_column_spacing(4);
    grid.set_row_spacing(4);
    grid.set_margin_bottom(FRAME_MARGIN);
    grid.set_margin_start(FRAME_MARGIN);
    grid.set_margin_end(FRAME_MARGIN);
    frame.set_child(Some(&grid));

    let mut row = 0;

    // Subscription controls
    let packet_id_label = gtk::Label::new(Some("Packet ID:"));
    packet_id_label.set_halign(gtk::Align::End);
    packet_id_label.set_margin_end(4);
    grid.attach(&packet_id_label, 0, row, 1, 1);
    let gtk_adjustment = gtk::Adjustment::new(1.0, 1.0, 65535.0, 1.0, 10.0, 1.0);
    let packet_id_entry = gtk::SpinButton::new(Some(&gtk_adjustment), 1.0, 0);
    packet_id_entry.set_tooltip_text(Some("Packet ID for the Subscription"));
    packet_id_entry.set_width_chars(6);
    grid.attach(&packet_id_entry, 1, row, 1, 1);
    let packet_id = Rc::new(RefCell::new(1u16));
    let packet_id_clone = Rc::clone(&packet_id);
    packet_id_entry.connect_value_changed(move |spin_button| {
        let value = spin_button.value() as u16;
        *packet_id_clone.borrow_mut() = value;
    });
    row += 1;

    // QoS level selection
    let qos_label = gtk::Label::new(Some("QoS:"));
    qos_label.set_halign(gtk::Align::End);
    qos_label.set_margin_end(4);
    grid.attach(&qos_label, 0, row, 1, 1);
    let qos_combo = gtk::ComboBoxText::new();
    qos_combo.append(Some("0"), "At Most Once (QoS 0)");
    qos_combo.append(Some("1"), "At Least Once (QoS 1)");
    qos_combo.append(Some("2"), "Exactly Once (QoS 2)");
    qos_combo.set_active(Some(0));
    qos_combo.set_tooltip_text(Some("Quality of Service for the Subscription"));
    grid.attach(&qos_combo, 1, row, 1, 1);
    row += 1;

    let topic_entry = gtk::Entry::new();
    topic_entry.set_placeholder_text(Some("Topic to subscribe to"));
    topic_entry.set_tooltip_text(Some("Topic to subscribe to"));
    topic_entry.set_width_chars(TOPIC_ENTRY_WIDTH_CHARS);
    grid.attach(&topic_entry, 0, row, 2, 1);
    row += 1;
    let subscribe_button = gtk::Button::with_label("Subscribe");
    subscribe_button.set_halign(gtk::Align::End);
    grid.attach(&subscribe_button, 1, row, 1, 1);

    let topic_entry_clone = topic_entry.clone();
    let qos_combo_clone = qos_combo.clone();
    subscribe_button.connect_clicked(move |_| {
        let topic = topic_entry_clone.text().to_string();
        let qos = match qos_combo_clone.active_text().as_deref() {
            Some("At Most Once (QoS 0)") => vaux_mqtt::QoSLevel::AtMostOnce,
            Some("At Least Once (QoS 1)") => vaux_mqtt::QoSLevel::AtLeastOnce,
            Some("Exactly Once (QoS 2)") => vaux_mqtt::QoSLevel::ExactlyOnce,
            _ => vaux_mqtt::QoSLevel::AtMostOnce, // Default fallback
        };
        let packet_id = *packet_id.borrow();
        // create a subscription filter for the settings
        if topic.is_empty() {
            println!("Topic is empty, cannot subscribe");
            return;
        }
        println!("Subscribing to topic: {} with QoS: {:?}", topic, qos);
        // create a subscribe command and send it
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        match rt.block_on(async {
            cmd_tx
                .send(client::Command::Subscribe(packet_id, qos, topic))
                .await
        }) {
            Ok(_) => {
                println!("Subscribe command sent");
            }
            Err(e) => {
                println!("Failed to send subscribe command: {e}");
            }
        }
    });

    frame
}

pub(crate) fn build_unsubscribe(cmd_tx: tokio::sync::mpsc::Sender<Command>) -> gtk::Frame {
    let frame = gtk::Frame::new(Some("Unsubscribe"));
    let grid = gtk::Grid::new();
    grid.set_column_spacing(4);
    grid.set_row_spacing(4);
    grid.set_margin_bottom(FRAME_MARGIN);
    grid.set_margin_start(FRAME_MARGIN);
    grid.set_margin_end(FRAME_MARGIN);
    frame.set_child(Some(&grid));

    let mut row = 0;

    // Unsubscription controls
    let packet_id_label = gtk::Label::new(Some("Packet ID:"));
    packet_id_label.set_halign(gtk::Align::End);
    packet_id_label.set_margin_end(4);
    grid.attach(&packet_id_label, 0, row, 1, 1);
    let gtk_adjustment = gtk::Adjustment::new(1.0, 1.0, 65535.0, 1.0, 10.0, 1.0);
    let packet_id_entry = gtk::SpinButton::new(Some(&gtk_adjustment), 1.0, 0);
    packet_id_entry.set_tooltip_text(Some("Packet ID for the Unsubscription"));
    packet_id_entry.set_width_chars(6);
    grid.attach(&packet_id_entry, 1, row, 1, 1);
    let packet_id = Rc::new(RefCell::new(1u16));
    let packet_id_clone = Rc::clone(&packet_id);
    packet_id_entry.connect_value_changed(move |spin_button| {
        let value = spin_button.value() as u16;
        *packet_id_clone.borrow_mut() = value;
    });
    row += 1;

    let topic_entry = gtk::Entry::new();
    topic_entry.set_placeholder_text(Some("Topic to unsubscribe from"));
    topic_entry.set_tooltip_text(Some("Topic to unsubscribe from"));
    topic_entry.set_width_chars(TOPIC_ENTRY_WIDTH_CHARS);
    grid.attach(&topic_entry, 0, row, 2, 1);
    row += 1;
    let unsubscribe_button = gtk::Button::with_label("Unsubscribe");
    unsubscribe_button.set_halign(gtk::Align::End);
    grid.attach(&unsubscribe_button, 1, row, 1, 1);

    let topic_entry_clone = topic_entry.clone();
    unsubscribe_button.connect_clicked(move |_| {
        let topic = topic_entry_clone.text().to_string();
        let packet_id = *packet_id.borrow();
        // create a subscription filter for the settings
        if topic.is_empty() {
            println!("Topic is empty, cannot unsubscribe");
            return;
        }
        println!("Unsubscribing from topic: {}", topic);
        // create an unsubscribe command and send it
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        match rt.block_on(async {
            cmd_tx
                .send(client::Command::Unsubscribe(packet_id, topic))
                .await
        }) {
            Ok(_) => {
                println!("Unsubscribe command sent");
            }
            Err(e) => {
                println!("Failed to send unsubscribe command: {e}");
            }
        }
    });
    frame
}

fn build_connect(
    ping: &gtk::Button,
    clean_start_check: &gtk::CheckButton,
    client_setting: &ClientSetting,
    cmd_tx: tokio::sync::mpsc::Sender<Command>,
) -> gtk::ToggleButton {
    let button = gtk::ToggleButton::with_label("Connect");
    let host = Rc::clone(&client_setting.host);
    let port = Rc::clone(&client_setting.port);
    let username = Rc::clone(&client_setting.username);
    let password = Rc::clone(&client_setting.password);
    let with_credentials = Rc::clone(&client_setting.with_credentials);
    let client_id = Rc::clone(&client_setting.client_id);
    let session_expiry = Rc::clone(&client_setting.session_expiry);
    let auto_ack = Rc::clone(&client_setting.auto_ack);
    let auto_packet_id = Rc::clone(&client_setting.auto_packet_id);
    let with_ping_resp = Rc::clone(&client_setting.with_ping_resp);
    let with_tls = Rc::clone(&client_setting.with_tls);
    let with_will = Rc::clone(&client_setting.with_will);
    let will_topic = Rc::clone(&client_setting.will_topic);
    let will_payload = Rc::clone(&client_setting.will_payload);
    let will_qos = Rc::clone(&client_setting.will_qos);
    let will_retain = Rc::clone(&client_setting.will_retain);
    let will_delay = Rc::clone(&client_setting.will_delay);
    let will_expiry = Rc::clone(&client_setting.will_expiry);

    let click_handler = clone!(
        #[weak]
        clean_start_check,
        #[weak]
        ping,
        move |b: &gtk::ToggleButton| {
            if b.is_active() {
                b.set_label("Disconnect");
                let mut connection = vaux_client::MqttConnection::new()
                    .with_host((*host).borrow().as_str())
                    .with_port(*port.borrow());
                if *with_tls.borrow() {
                    connection = connection.with_tls();
                }
                if *with_credentials.borrow() {
                    connection = connection
                        .with_credentials(username.borrow().as_str(), password.borrow().as_str());
                }

                if clean_start_check.is_active() {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();
                    match rt.block_on(async {
                        cmd_tx
                            .send(client::Command::ResumeSession(connection))
                            .await
                    }) {
                        Ok(_) => {
                            println!("MQTT Client connected successfully");
                            ping.set_sensitive(true);
                        }
                        Err(e) => {
                            println!("Failed to connect MQTT Client: {e}");
                            b.set_active(false); // Reset button state on failure
                            return;
                        }
                    }
                } else {
                    let mut builder = vaux_client::ClientBuilder::new(connection)
                        .with_client_id((*client_id).borrow().as_str())
                        .with_session_expiry(Duration::from_secs(*session_expiry.borrow() as u64))
                        .with_auto_ack(*auto_ack.borrow())
                        .with_auto_packet_id(*auto_packet_id.borrow())
                        .with_pingresp(*with_ping_resp.borrow());
                    if *with_will.borrow() {
                        let mut will_msg =
                            WillMessage::new(*will_qos.borrow(), *will_retain.borrow());
                        will_msg.topic = (*will_topic).borrow().to_string();
                        will_msg.payload = (*will_payload).borrow().as_bytes().to_vec();
                        will_msg.set_delay(*will_delay.borrow());
                        will_msg.set_expiry(*will_expiry.borrow());
                        builder = builder.with_will_message(will_msg);
                    };

                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();
                    match rt.block_on(async {
                        cmd_tx.send(client::Command::StartClient(builder)).await
                    }) {
                        Ok(_) => {
                            println!("MQTT Client connected successfully");
                            ping.set_sensitive(true);
                        }
                        Err(e) => {
                            println!("Failed to connect MQTT Client: {e}");
                            b.set_active(false); // Reset button state on failure
                            return;
                        }
                    }
                }
            } else {
                b.set_label("Connect");
                ping.set_sensitive(false);
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                match rt.block_on(async { cmd_tx.send(client::Command::StopClient).await }) {
                    Ok(_) => {
                        println!("MQTT Client disconnected successfully");
                    }
                    Err(e) => {
                        println!("Failed to disconnect MQTT Client: {e}");
                    }
                }
                // Reset the clean start checkbox
                clean_start_check.set_sensitive(true);
            }
        }
    );
    button.connect_clicked(click_handler);

    button
}

fn build_publish(cmd_tx: tokio::sync::mpsc::Sender<Command>) -> gtk::Frame {
    let frame = gtk::Frame::new(Some("Publish"));

    // frame grid layout
    let grid = gtk::Grid::new();
    grid.set_column_spacing(4);
    grid.set_row_spacing(4);
    grid.set_margin_bottom(FRAME_MARGIN);
    grid.set_margin_start(FRAME_MARGIN);
    grid.set_margin_end(FRAME_MARGIN);
    frame.set_child(Some(&grid));
    // packet id
    let label = gtk::Label::new(Some("Packet ID:"));
    label.set_halign(gtk::Align::End);
    label.set_margin_end(4);
    grid.attach(&label, 0, 0, 1, 1);
    let gtk_adjustment = gtk::Adjustment::new(0.0, 0.0, 65535.0, 1.0, 10.0, 1.0);
    let packet_id_entry = gtk::SpinButton::new(Some(&gtk_adjustment), 1.0, 0);
    packet_id_entry.set_tooltip_text(Some("Packet ID for the Publish"));
    packet_id_entry.set_width_chars(6);
    grid.attach(&packet_id_entry, 1, 0, 1, 1);
    // qos
    let qos_label = gtk::Label::new(Some("QoS:"));
    qos_label.set_halign(gtk::Align::End);
    qos_label.set_margin_end(4);
    grid.attach(&qos_label, 0, 1, 1, 1);
    let qos_combo = gtk::ComboBoxText::new();
    qos_combo.append(Some("0"), "At Most Once (QoS 0)");
    qos_combo.append(Some("1"), "At Least Once (QoS 1)");
    qos_combo.append(Some("2"), "Exactly Once (QoS 2)");
    qos_combo.set_active(Some(0));
    qos_combo.set_tooltip_text(Some("Quality of Service for the Publish"));
    grid.attach(&qos_combo, 1, 1, 1, 1);
    // retain
    let retain_check = gtk::CheckButton::with_label("Retain");
    retain_check.set_tooltip_text(Some("Retain the message on the broker"));
    grid.attach(&retain_check, 2, 1, 1, 1);
    // topic
    let topic_entry = gtk::Entry::new();
    topic_entry.set_placeholder_text(Some("Topic to publish to"));
    topic_entry.set_tooltip_text(Some("Topic to publish to"));
    topic_entry.set_width_chars(TOPIC_ENTRY_WIDTH_CHARS);
    grid.attach(&topic_entry, 0, 2, 3, 1);
    // message
    let message_entry = gtk::TextView::new();
    message_entry.set_tooltip_text(Some("Message payload to publish"));
    message_entry.set_wrap_mode(gtk::WrapMode::Word);
    message_entry.set_size_request(PUBLISH_TEXT_WIDTH_REQUEST, PUBLISH_TEXT_HEIGHT_REQUEST);
    grid.attach(&message_entry, 0, 3, 3, 1);
    // publish button
    let publish_button = gtk::Button::with_label("Publish");
    publish_button.set_halign(gtk::Align::End);
    grid.attach(&publish_button, 2, 4, 1, 1);
    // publish button click handler
    publish_button.connect_clicked(move |_| {
        let topic = topic_entry.text().to_string();
        let buffer = message_entry.buffer();
        let message = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
        let qos = match qos_combo.active_text().as_deref() {
            Some("At Most Once (QoS 0)") => vaux_mqtt::QoSLevel::AtMostOnce,
            Some("At Least Once (QoS 1)") => vaux_mqtt::QoSLevel::AtLeastOnce,
            Some("Exactly Once (QoS 2)") => vaux_mqtt::QoSLevel::ExactlyOnce,
            _ => vaux_mqtt::QoSLevel::AtMostOnce, // Default fallback
        };
        let retain = retain_check.is_active();
        let packet_id = if packet_id_entry.value() as u16 == 0 {
            None
        } else {
            Some(packet_id_entry.value() as u16)
        };
        if topic.is_empty() {
            println!("Topic is empty, cannot publish");
            return;
        }
        println!(
            "Publishing to topic: {} with QoS: {:?}, retain: {}, packet_id: {:?}, message: {}",
            topic, qos, retain, packet_id, message
        );
        let publish =
            vaux_mqtt::publish::Publish::new_with_message(packet_id, topic, qos, message.as_str())
                .and_then(|p| Ok(p.with_retain(retain)));
        if publish.is_err() {
            println!(
                "Failed to create publish packet: {}",
                publish.err().unwrap()
            );
            return;
        }
        let publish = publish.unwrap();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        match rt.block_on(async { cmd_tx.send(client::Command::Publish(publish)).await }) {
            Ok(_) => {
                println!("Publish command sent");
            }
            Err(e) => {
                println!("Failed to send publish command: {e}");
            }
        }
    });

    frame
}

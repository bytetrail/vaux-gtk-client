use std::{cell::RefCell, rc::Rc};

use gtk4::{self as gtk};

use glib_macros::clone;
use gtk::prelude::*;

use crate::client::{ClientSetting, Command};

const FRAME_MARGIN: i32 = 6;
const FILE_ENTRY_WIDTH_CHARS: i32 = 50;
const TOPIC_ENTRY_WIDTH_CHARS: i32 = 50;
const MESSAGE_TEXT_WIDTH_REQUEST: i32 = 300;
const MESSAGE_TEXT_HEIGHT_REQUEST: i32 = 120;

const WILL_DELAY_MIN: f64 = 0.0; // 1 second
const WILL_DELAY_MAX: f64 = 120.0; // 2 minutes
const WILL_EXPIRY_MIN: f64 = 0.0; // 0 seconds
const WILL_EXPIRY_MAX: f64 = 3600.0; // 1 hour

pub fn build_will(client_setting: &ClientSetting) -> gtk::Frame {
    let frame = gtk::Frame::new(Some("Will Message"));

    let grid = gtk::Grid::new();
    grid.set_column_spacing(4);
    grid.set_row_spacing(4);
    grid.set_margin_bottom(FRAME_MARGIN);
    grid.set_margin_start(FRAME_MARGIN);
    grid.set_margin_end(FRAME_MARGIN);

    let mut row = 0;

    let will_enable = Rc::clone(&client_setting.with_will);
    let will_enable_toggle = gtk::CheckButton::new();
    will_enable_toggle.set_active(*will_enable.borrow());
    will_enable_toggle.set_label(Some("Enable Will Message"));
    will_enable_toggle.set_active(*will_enable.borrow());
    will_enable_toggle.set_tooltip_text(Some("Enable or disable the will message"));
    grid.attach(&will_enable_toggle, 0, row, 2, 1);
    row += 1;

    let will_qos_combo = gtk::ComboBoxText::new();
    will_qos_combo.append(Some("0"), "At Most Once (QoS 0)");
    will_qos_combo.append(Some("1"), "At Least Once (QoS 1)");
    will_qos_combo.append(Some("2"), "Exactly Once (QoS 2)");
    will_qos_combo.set_active(Some(0));
    will_qos_combo.set_tooltip_text(Some("Quality of Service for the Will Message"));
    will_qos_combo.set_sensitive(*will_enable.borrow());
    let will_qos = Rc::clone(&client_setting.will_qos);
    will_qos_combo.connect_changed(move |combo: &gtk::ComboBoxText| {
        if let Some(active) = combo.active_text() {
            let qos_value = match active.as_str() {
                "At Most Once (QoS 0)" => vaux_mqtt::QoSLevel::AtMostOnce,
                "At Least Once (QoS 1)" => vaux_mqtt::QoSLevel::AtLeastOnce,
                "Exactly Once (QoS 2)" => vaux_mqtt::QoSLevel::ExactlyOnce,
                _ => vaux_mqtt::QoSLevel::AtMostOnce, // Default fallback
            };
            *(*will_qos).borrow_mut() = qos_value;
        }
    });
    grid.attach(&will_qos_combo, 0, row, 2, 1);
    row += 1;

    let will_retain_toggle = gtk::CheckButton::new();
    will_retain_toggle.set_label(Some("Retain Will Message"));
    will_retain_toggle.set_active(false);
    will_retain_toggle.set_tooltip_text(Some("Retain the Will Message on the broker"));
    will_retain_toggle.set_sensitive(*will_enable.borrow());
    let will_retain = Rc::clone(&client_setting.will_retain);
    will_retain_toggle.connect_toggled(move |b: &gtk::CheckButton| {
        *(*will_retain).borrow_mut() = b.is_active();
    });
    grid.attach(&will_retain_toggle, 0, row, 2, 1);
    row += 1;

    let label = gtk::Label::new(Some("Will Topic:"));
    grid.attach(&label, 0, row, 1, 1);

    let will_topic_entry = gtk::Entry::new();
    will_topic_entry.set_placeholder_text(Some("Will Topic"));
    will_topic_entry.set_tooltip_text(Some("Topic for the Will Message"));
    will_topic_entry.set_width_chars(TOPIC_ENTRY_WIDTH_CHARS);
    will_topic_entry.set_sensitive(*will_enable.borrow());
    let will_topic = Rc::clone(&client_setting.will_topic);
    will_topic_entry.connect_changed(move |entry| {
        *(*will_topic).borrow_mut() = entry.text().to_string();
    });
    grid.attach(&will_topic_entry, 1, row, 1, 1);
    row += 1;

    let label = gtk::Label::new(Some("Will Payload:"));
    grid.attach(&label, 0, row, 1, 1);
    row += 1;

    let will_payload_text = gtk::TextView::new();
    will_payload_text.set_tooltip_text(Some("Payload for the Will Message"));
    will_payload_text.set_sensitive(*will_enable.borrow());
    will_payload_text.set_size_request(MESSAGE_TEXT_WIDTH_REQUEST, MESSAGE_TEXT_HEIGHT_REQUEST);
    will_payload_text.set_wrap_mode(gtk::WrapMode::Word);
    let will_payload = Rc::clone(&client_setting.will_payload);
    will_payload_text.buffer().connect_changed(move |b| {
        let start = b.start_iter();
        let end = b.end_iter();
        let text = b.text(&start, &end, false);
        *(*will_payload).borrow_mut() = text.to_string();
    });
    let will_payload_scrolled = gtk::ScrolledWindow::new();
    will_payload_scrolled.set_child(Some(&will_payload_text));
    will_payload_scrolled.set_min_content_width(MESSAGE_TEXT_WIDTH_REQUEST);
    will_payload_scrolled.set_min_content_height(MESSAGE_TEXT_HEIGHT_REQUEST);
    will_payload_scrolled.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    grid.attach(&will_payload_scrolled, 0, row, 2, 1);
    row += 1;
    // add the delay entry
    let will_delay_label = gtk::Label::new(Some("Will Delay (seconds):"));
    will_delay_label.set_halign(gtk::Align::End);
    will_delay_label.set_margin_end(4);
    grid.attach(&will_delay_label, 0, row, 1, 1);
    let gtk_adjustment = gtk::Adjustment::new(
        *client_setting.will_delay.borrow() as f64,
        WILL_DELAY_MIN,
        WILL_DELAY_MAX,
        1.0,
        10.0,
        1.0,
    );
    let will_delay_entry = gtk::SpinButton::new(Some(&gtk_adjustment), 1.0, 0);
    will_delay_entry.set_value(*client_setting.will_delay.borrow() as f64);
    will_delay_entry.set_tooltip_text(Some("Delay before the Will Message is sent"));
    will_delay_entry.set_sensitive(*will_enable.borrow());
    let will_delay = Rc::clone(&client_setting.will_delay);
    will_delay_entry.connect_value_changed(move |spin_button| {
        let value = spin_button.value() as u32;
        *(*will_delay).borrow_mut() = value;
    });
    grid.attach(&will_delay_entry, 1, row, 1, 1);
    row += 1;
    // add the expiry entry
    let will_expiry_label = gtk::Label::new(Some("Will Expiry (seconds):"));
    will_expiry_label.set_halign(gtk::Align::End);
    will_expiry_label.set_margin_end(4);
    grid.attach(&will_expiry_label, 0, row, 1, 1);
    let gtk_adjustment = gtk::Adjustment::new(
        *client_setting.will_expiry.borrow() as f64,
        WILL_EXPIRY_MIN,
        WILL_EXPIRY_MAX,
        1.0,
        10.0,
        1.0,
    );
    let will_expiry_entry = gtk::SpinButton::new(Some(&gtk_adjustment), 1.0, 0);
    will_expiry_entry.set_value(*client_setting.will_expiry.borrow() as f64);
    will_expiry_entry.set_tooltip_text(Some("Expiry time for the Will Message"));
    will_expiry_entry.set_sensitive(*will_enable.borrow());
    let will_expiry = Rc::clone(&client_setting.will_expiry);
    will_expiry_entry.connect_value_changed(move |spin_button| {
        let value = spin_button.value() as u32;
        *(*will_expiry).borrow_mut() = value;
    });
    grid.attach(&will_expiry_entry, 1, row, 1, 1);

    let will_toggle_handler = clone!(
        #[weak]
        will_qos_combo,
        #[weak]
        will_retain_toggle,
        #[weak]
        will_topic_entry,
        #[weak]
        will_payload_text,
        #[weak]
        will_delay_entry,
        #[weak]
        will_expiry_entry,
        move |b: &gtk::CheckButton| {
            println!("Will Message toggled: {}", b.is_active());
            *(*will_enable).borrow_mut() = b.is_active();
            will_qos_combo.set_sensitive(b.is_active());
            will_retain_toggle.set_sensitive(b.is_active());
            will_topic_entry.set_sensitive(b.is_active());
            will_payload_text.set_sensitive(b.is_active());
            will_delay_entry.set_sensitive(b.is_active());
            will_expiry_entry.set_sensitive(b.is_active());
        }
    );
    will_enable_toggle.connect_toggled(will_toggle_handler);

    frame.set_child(Some(&grid));

    frame
}

pub fn build_settings(client_setting: &ClientSetting) -> (gtk::Frame, gtk::CheckButton) {
    // Grid for MQTT client settings controls

    let frame = gtk::Frame::new(Some("Client Settings"));

    let grid = gtk::Grid::new();
    grid.set_row_spacing(4);
    grid.set_margin_start(FRAME_MARGIN);
    grid.set_margin_end(FRAME_MARGIN);
    grid.set_margin_top(FRAME_MARGIN);
    grid.set_margin_bottom(FRAME_MARGIN);
    frame.set_child(Some(&grid));

    let mut row = 0;

    let clean_start_check = gtk::CheckButton::new();
    clean_start_check.set_label(Some("Resume Session"));
    clean_start_check.set_halign(gtk::Align::Start);
    clean_start_check.set_tooltip_text(Some(
        "Resume prior session when present, otherwise clean start",
    ));
    clean_start_check.set_sensitive((*client_setting.clean_start.borrow()).is_some());
    let _clean_start = Rc::clone(&client_setting.clean_start);
    clean_start_check.connect_toggled(move |button: &gtk::CheckButton| {
        if button.is_active() {
            *(*_clean_start).borrow_mut() = Some(true);
        } else {
            *(*_clean_start).borrow_mut() = Some(false);
        }
    });
    grid.attach(&clean_start_check, 1, row, 1, 1);
    row += 1;

    let label = gtk::Label::new(Some("Client ID:"));
    label.set_halign(gtk4::Align::End);
    label.set_margin_end(4);
    let client_id_entry = gtk::Entry::new();
    client_id_entry.set_placeholder_text(Some(client_setting.client_id.borrow().as_str()));
    let _client_id = Rc::clone(&client_setting.client_id);
    client_id_entry.connect_changed(move |entry| {
        *(*_client_id).borrow_mut() = entry.text().to_string();
    });
    grid.attach(&label, 0, row, 1, 1);
    grid.attach(&client_id_entry, 1, row, 1, 1);
    row += 1;

    // host
    let label = gtk::Label::new(Some("Host:"));
    label.set_halign(gtk4::Align::End);
    label.set_margin_end(4);
    let host_entry = gtk::Entry::new();
    host_entry.set_text("localhost");
    let _host = Rc::clone(&client_setting.host);
    host_entry.connect_changed(move |entry| {
        *(*_host).borrow_mut() = entry.text().to_string();
    });
    grid.attach(&label, 0, row, 1, 1);
    grid.attach(&host_entry, 1, row, 1, 1);
    row += 1;

    // port
    let label = gtk::Label::new(Some("Port:"));
    label.set_halign(gtk4::Align::End);
    label.set_margin_end(4);
    let gtk_adjustment = gtk::Adjustment::new(1883.0, 1883.0, 65536.0, 1.0, 10.0, 1.0);
    let port_entry = gtk::SpinButton::new(Some(&gtk_adjustment), 1.0, 0);
    port_entry.set_value(*client_setting.port.borrow() as f64);
    let _port = Rc::clone(&client_setting.port);
    port_entry.connect_value_changed(move |spin_button| {
        let value = spin_button.value() as u16;
        *(*_port).borrow_mut() = value;
    });
    grid.attach(&label, 0, row, 1, 1);
    grid.attach(&port_entry, 1, row, 1, 1);
    row += 1;

    let label = gtk::Label::new(Some("Session Expiration:"));
    label.set_halign(gtk4::Align::End);
    let gtk_adjustment = gtk::Adjustment::new(60.0, 30.0, 1200.0, 1.0, 15.0, 1.0);
    let session_expiry_entry = gtk::SpinButton::new(Some(&gtk_adjustment), 1.0, 0);
    session_expiry_entry.set_value(*client_setting.session_expiry.borrow() as f64);
    session_expiry_entry.set_tooltip_text(Some("Session expiration time in seconds"));
    let _session_expiry = Rc::clone(&client_setting.session_expiry);
    session_expiry_entry.connect_value_changed(move |spin_button| {
        let value = spin_button.value() as u32;
        *(*_session_expiry).borrow_mut() = value;
    });
    session_expiry_entry.set_margin_end(4);
    grid.attach(&label, 0, row, 1, 1);
    grid.attach(&session_expiry_entry, 1, row, 1, 1);
    row += 1;

    let auto_ack = gtk::CheckButton::with_label("Enable Auto Ack");
    auto_ack.set_halign(gtk::Align::Start);
    auto_ack.set_tooltip_text(Some("Enable automatic acknowledgment of MQTT messages"));
    auto_ack.set_active(*client_setting.auto_ack.borrow());
    let _auto_ack = Rc::clone(&client_setting.auto_ack);
    auto_ack.connect_toggled(move |button| {
        *(*_auto_ack).borrow_mut() = button.is_active();
    });
    grid.attach(&auto_ack, 1, row, 1, 1);
    row += 1;
    let auto_packet_id = gtk::CheckButton::with_label("Enable Auto Packet ID");
    auto_packet_id.set_halign(gtk::Align::Start);
    auto_packet_id.set_tooltip_text(Some(
        "Enable automatic generation of packet IDs for MQTT messages",
    ));
    auto_packet_id.set_active(*client_setting.auto_packet_id.borrow());
    let _auto_packet_id = Rc::clone(&client_setting.auto_packet_id);
    auto_packet_id.connect_toggled(move |button| {
        *(*_auto_packet_id).borrow_mut() = button.is_active();
    });
    grid.attach(&auto_packet_id, 1, row, 1, 1);
    row += 1;

    let with_ping_resp = gtk::CheckButton::with_label("Ping Response Pass-through");
    with_ping_resp.set_halign(gtk::Align::Start);
    with_ping_resp.set_tooltip_text(Some("Enable PINGRESP in the message log"));
    with_ping_resp.set_active(*client_setting.with_ping_resp.borrow());
    let _with_ping_resp = Rc::clone(&client_setting.with_ping_resp);
    with_ping_resp.connect_toggled(move |button| {
        *(*_with_ping_resp).borrow_mut() = button.is_active();
    });
    grid.attach(&with_ping_resp, 1, row, 1, 1);

    (frame, clean_start_check)
}

pub fn build_tls(
    with_tls: Rc<RefCell<bool>>,
    ca_file: Rc<RefCell<String>>,
    client_cert: Rc<RefCell<String>>,
) -> gtk::Frame {
    let frame = gtk::Frame::new(Some("TLS"));
    let grid = gtk::Grid::new();
    grid.set_column_spacing(4);
    grid.set_row_spacing(4);
    grid.set_margin_bottom(FRAME_MARGIN);
    grid.set_margin_start(FRAME_MARGIN);
    grid.set_margin_end(FRAME_MARGIN);

    let tls_check = gtk::CheckButton::with_label("Enable TLS");
    let tls_ca_entry = gtk::Entry::new();
    let tls_client_entry = gtk::Entry::new();
    let ca_file_chooser_button = gtk::Button::with_label("File...");
    let client_file_chooser_button = gtk::Button::with_label("File...");

    tls_check.set_halign(gtk::Align::Start);
    tls_check.set_tooltip_text(Some("Enable TLS for MQTT connection"));
    tls_check.set_active(false);
    tls_check.set_margin_bottom(8);
    let _with_tls = Rc::clone(&with_tls);
    let with_tls_toggle_handler = clone!(
        #[weak]
        tls_ca_entry,
        #[weak]
        tls_client_entry,
        #[weak]
        ca_file_chooser_button,
        #[weak]
        client_file_chooser_button,
        move |button: &gtk::CheckButton| {
            println!("TLS toggled: {}", button.is_active());
            *(*_with_tls).borrow_mut() = button.is_active();
            if button.is_active() {
                tls_ca_entry.set_sensitive(true);
                tls_client_entry.set_sensitive(true);
                ca_file_chooser_button.set_sensitive(true);
                client_file_chooser_button.set_sensitive(true);
            } else {
                tls_ca_entry.set_sensitive(false);
                tls_client_entry.set_sensitive(false);
                ca_file_chooser_button.set_sensitive(false);
                client_file_chooser_button.set_sensitive(false);
            }
        }
    );
    tls_check.connect_toggled(with_tls_toggle_handler);
    grid.attach(&tls_check, 0, 0, 2, 1);

    let label = gtk::Label::new(Some("CA Certificate:"));
    label.set_halign(gtk4::Align::End);
    grid.attach(&label, 0, 1, 1, 1);
    tls_ca_entry.set_placeholder_text(Some("CA Certificate Path"));
    tls_ca_entry.set_sensitive(*with_tls.borrow());
    tls_ca_entry.set_width_chars(FILE_ENTRY_WIDTH_CHARS);
    tls_ca_entry.connect_changed(move |entry| {
        *(*ca_file).borrow_mut() = entry.text().to_string();
    });
    grid.attach(&tls_ca_entry, 1, 1, 1, 1);
    ca_file_chooser_button.set_halign(gtk::Align::Start);
    ca_file_chooser_button.set_sensitive(*with_tls.borrow());
    grid.attach(&ca_file_chooser_button, 2, 1, 1, 1);

    let label = gtk::Label::new(Some("Client Certificate:"));
    label.set_halign(gtk4::Align::End);
    grid.attach(&label, 0, 2, 1, 1);
    tls_client_entry.set_placeholder_text(Some("Client Certificate Path"));
    tls_client_entry.set_sensitive(*with_tls.borrow());
    tls_client_entry.set_width_chars(FILE_ENTRY_WIDTH_CHARS);
    tls_client_entry.connect_changed(move |entry| {
        *(*client_cert).borrow_mut() = entry.text().to_string();
    });
    grid.attach(&tls_client_entry, 1, 2, 1, 1);
    client_file_chooser_button.set_halign(gtk::Align::Start);
    client_file_chooser_button.set_sensitive(*with_tls.borrow());
    grid.attach(&client_file_chooser_button, 2, 2, 1, 1);

    frame.set_child(Some(&grid));
    frame
}

pub fn build_credentials(
    with_cred: Rc<RefCell<bool>>,
    username: Rc<RefCell<String>>,
    password: Rc<RefCell<String>>,
) -> gtk::Frame {
    let frame = gtk::Frame::new(Some("Credentials"));
    let grid = gtk::Grid::new();
    grid.set_column_spacing(4);
    grid.set_row_spacing(4);
    grid.set_margin_bottom(FRAME_MARGIN);
    grid.set_margin_start(FRAME_MARGIN);
    grid.set_margin_end(FRAME_MARGIN);

    let username_entry = gtk::Entry::new();
    let password_entry = gtk::Entry::new();

    let cred_check = gtk::CheckButton::with_label("Use Credentials");
    cred_check.set_halign(gtk::Align::Start);
    cred_check.set_active(*with_cred.borrow());
    cred_check.set_tooltip_text(Some(
        "Enable to use username and password for authentication",
    ));
    cred_check.set_margin_bottom(8);
    let _with_cred = Rc::clone(&with_cred);

    let toggle_handler = clone!(
        #[weak]
        username_entry,
        #[weak]
        password_entry,
        move |button: &gtk::CheckButton| {
            println!("Credentials toggled: {}", button.is_active());
            *(*_with_cred).borrow_mut() = button.is_active();
            if button.is_active() {
                username_entry.set_sensitive(true);
                password_entry.set_sensitive(true);
            } else {
                username_entry.set_sensitive(false);
                password_entry.set_sensitive(false);
            }
        }
    );
    cred_check.connect_toggled(toggle_handler);
    grid.attach(&cred_check, 0, 0, 2, 1);

    let label = gtk::Label::new(Some("Username:"));
    label.set_halign(gtk4::Align::End);
    grid.attach(&label, 0, 1, 1, 1);
    username_entry.set_placeholder_text(Some("username"));
    username_entry.set_sensitive(*with_cred.borrow());
    username_entry.connect_changed(move |entry| {
        *username.borrow_mut() = entry.text().to_string();
        println!("Username changed: {}", *username.borrow());
    });
    grid.attach(&username_entry, 1, 1, 1, 1);
    let label = gtk::Label::new(Some("Password:"));
    label.set_halign(gtk4::Align::End);
    grid.attach(&label, 0, 2, 1, 1);
    password_entry.set_placeholder_text(Some("password"));
    password_entry.set_visibility(false);
    password_entry.set_sensitive(*with_cred.borrow());
    password_entry.connect_changed(move |entry| {
        *password.borrow_mut() = entry.text().to_string();
    });
    grid.attach(&password_entry, 1, 2, 1, 1);

    frame.set_child(Some(&grid));
    frame
}

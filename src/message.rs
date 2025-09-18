use std::{cell::RefCell, rc::Rc};

use gtk::prelude::*;
use gtk4::{
    self as gtk,
    gio::{self},
};

use crate::model::PacketObject;

pub fn build_message_view(message_model: Rc<RefCell<gio::ListStore>>) -> gtk::Frame {
    let frame = gtk::Frame::new(Some("Messages"));

    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scrolled_window.set_hexpand(true);
    scrolled_window.set_vexpand(true);

    // create a model for the message view
    //let message_model = gio::ListStore::new::<PacketObject>();

    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_, item| {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        let packet_label = gtk::Label::new(None);
        packet_label.set_hexpand(false);
        hbox.append(&packet_label);
        let id_label = gtk::Label::new(None);
        id_label.set_margin_start(10);
        id_label.set_hexpand(true);
        hbox.append(&id_label);

        item.downcast_ref::<gtk::ListItem>()
            .expect("Failed to downcast")
            .set_child(Some(&hbox));
    });
    factory.connect_bind(move |_, item| {
        let packet = item
            .downcast_ref::<gtk::ListItem>()
            .expect("Failed to downcast")
            .item()
            .and_downcast::<PacketObject>()
            .expect("Failed to downcast to PacketObject");

        let hbox = item
            .downcast_ref::<gtk::ListItem>()
            .expect("Failed to downcast")
            .child()
            .and_downcast::<gtk::Box>()
            .expect("Failed to downcast to Box");

        let packet_label = hbox
            .first_child()
            .and_downcast::<gtk::Label>()
            .expect("Failed to get packet_label");
        let id_label = hbox
            .last_child()
            .and_downcast::<gtk::Label>()
            .expect("Failed to get id_label");

        let mut id_str = String::new();
        packet_label.set_text(&packet.packet_type().to_string());
        id_label.set_text(if &packet.packet_id() == &0 {
            "-"
        } else {
            id_str = packet.packet_id().to_string();
            id_str.as_str()
        });
    });

    let model = (*message_model.clone()).borrow().clone();
    let selection_model = gtk::SingleSelection::new(Some(model));
    let list_view = gtk::ListView::new(Some(selection_model), Some(factory));
    list_view.set_hexpand(true);

    scrolled_window.set_child(Some(&list_view));
    frame.set_child(Some(&scrolled_window));

    frame
}

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
        let label = gtk::Label::new(None);
        item.downcast_ref::<gtk::ListItem>()
            .expect("Failed to downcast")
            .set_child(Some(&label));
    });
    factory.connect_bind(move |_, item| {
        let packet = item
            .downcast_ref::<gtk::ListItem>()
            .expect("Failed to downcast")
            .item()
            .and_downcast::<PacketObject>()
            .expect("Failed to downcast to PacketObject");

        let label = item
            .downcast_ref::<gtk::ListItem>()
            .expect("Failed to downcast")
            .child()
            .and_downcast::<gtk::Label>()
            .expect("Failed to downcast to Label");

        label.set_text(&packet.packet_type().to_string());
    });

    let foo = (*message_model.clone()).borrow().clone();
    let selection_model = gtk::SingleSelection::new(Some(foo));
    let list_view = gtk::ListView::new(Some(selection_model), Some(factory));
    list_view.set_hexpand(true);

    scrolled_window.set_child(Some(&list_view));
    frame.set_child(Some(&scrolled_window));

    frame
}

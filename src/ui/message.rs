use std::{cell::RefCell, rc::Rc};

use gtk::prelude::*;
use gtk4::{
    self as gtk,
    gio::{self},
};

use crate::model::PacketObject;

pub fn build_message_view(message_model: Rc<RefCell<gio::ListStore>>) -> gtk::Frame {

    let frame = gtk::Frame::new(Some("Messages"));
    // Create a vertical box to hold headers and the list
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    // Create header row with direction column
    let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    let direction_header = gtk::Label::new(Some(""));
    direction_header.set_hexpand(false);
    direction_header.set_margin_bottom(4);
    direction_header.set_markup("<b>Dir</b>");
    header_box.append(&direction_header);
    let packet_header = gtk::Label::new(Some("Type"));
    packet_header.set_hexpand(false);
    packet_header.set_margin_bottom(4);
    packet_header.set_markup("<b>Type</b>");
    header_box.append(&packet_header);
    let timestamp_header = gtk::Label::new(Some("Timestamp"));
    timestamp_header.set_hexpand(false);
    timestamp_header.set_margin_bottom(4);
    timestamp_header.set_markup("<b>Timestamp</b>");
    header_box.append(&timestamp_header);
    let id_header = gtk::Label::new(Some("ID"));
    id_header.set_hexpand(true);
    id_header.set_margin_start(10);
    id_header.set_margin_bottom(4);
    id_header.set_markup("<b>ID</b>");
    header_box.append(&id_header);

    vbox.append(&header_box);

    let scrolled_window = gtk::ScrolledWindow::new();
    scrolled_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scrolled_window.set_hexpand(true);
    scrolled_window.set_vexpand(true);

    // create a model for the message view
    //let message_model = gio::ListStore::new::<PacketObject>();

    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_, item| {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        // Direction indicator
        let direction_icon = gtk::DrawingArea::new();
        direction_icon.set_content_width(24);
        direction_icon.set_content_height(18);
        hbox.append(&direction_icon);
        let packet_label = gtk::Label::new(None);
        packet_label.set_hexpand(false);
        hbox.append(&packet_label);
        let timestamp_label = gtk::Label::new(None);
        timestamp_label.set_hexpand(false);
        hbox.append(&timestamp_label);
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

        // Direction icon is the first child
        let direction_icon = hbox
            .first_child()
            .and_downcast::<gtk::DrawingArea>()
            .expect("Failed to get direction_icon");

        // Draw the direction icon
        let exchange = packet.exchange();
        direction_icon.set_draw_func(move |_, cr, width, height| {
            use gtk4::gdk::RGBA;
            use std::f64::consts::PI;
            // Layout constants
            let bar_x = 4.0;
            let arrow_y = height as f64 / 2.0;
            let arrow_len = width as f64 - bar_x - 8.0;
            let head_size = 8.0;

            // Draw vertical bar
            cr.set_source_rgba(0.3, 0.3, 0.3, 1.0);
            cr.set_line_width(2.0);
            cr.move_to(bar_x, 2.0);
            cr.line_to(bar_x, height as f64 - 2.0);
            cr.stroke().unwrap();

            if exchange == "send" {
                // Arrowhead to the right of the bar, pointing left
                cr.set_source_rgba(0.0, 0.5, 1.0, 1.0); // blue
                cr.set_line_width(2.0);
                // Arrowhead point at bar_x, wide ends to the right
                let arrow_tip_x = bar_x + arrow_len;
                let arrow_base_x = bar_x + 2.0;
                // Shaft (hidden, just head)
                // Arrowhead
                cr.move_to(arrow_base_x, arrow_y);
                cr.line_to(
                    arrow_tip_x, arrow_y,
                );
                cr.move_to(arrow_tip_x, arrow_y);
                cr.line_to(
                    arrow_tip_x - head_size * (-(PI / 6.0)).cos(),
                    arrow_y - head_size * (-(PI / 6.0)).sin(),
                );
                cr.move_to(arrow_tip_x, arrow_y);
                cr.line_to(
                    arrow_tip_x - head_size * (-(PI / 6.0)).cos(),
                    arrow_y + head_size * (-(PI / 6.0)).sin(),
                );
                cr.stroke().unwrap();
            } else {
                // Arrowhead to the right of the bar, pointing left
                cr.set_source_rgba(0.0, 1.0, 0.5, 1.0); // blue
                cr.set_line_width(2.0);
                // Arrowhead point at bar_x, wide ends to the right
                let arrow_tip_x = bar_x + 2.0;
                let arrow_base_x = bar_x + arrow_len;
                cr.move_to(arrow_base_x, arrow_y);
                cr.line_to(
                    arrow_tip_x, arrow_y,
                );
                cr.move_to(arrow_tip_x, arrow_y);
                cr.line_to(
                    arrow_tip_x + head_size * (-(PI / 6.0)).cos(),
                    arrow_y - head_size * (-(PI / 6.0)).sin(),
                );
                cr.move_to(arrow_tip_x, arrow_y);
                cr.line_to(
                    arrow_tip_x + head_size * (-(PI / 6.0)).cos(),
                    arrow_y + head_size * (-(PI / 6.0)).sin(),
                );
                cr.stroke().unwrap();
            }
        });

        let packet_label = direction_icon
            .next_sibling()
            .and_downcast::<gtk::Label>()
            .expect("Failed to get packet_label");

        let timestamp_label = packet_label
            .next_sibling()
            .and_downcast::<gtk::Label>()
            .expect("Failed to get timestamp_label");

        let id_label = hbox
            .last_child()
            .and_downcast::<gtk::Label>()
            .expect("Failed to get id_label");

        let id_str: String;
        packet_label.set_text(&packet.packet_type().to_string());
        timestamp_label.set_text(&packet.timestamp().to_string());   
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
    vbox.append(&scrolled_window);
    frame.set_child(Some(&vbox));

    frame
}

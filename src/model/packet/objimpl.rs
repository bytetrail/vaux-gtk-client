use glib::{
    object::ObjectExt,
    subclass::{object::ObjectImpl, types::ObjectSubclass},
};
use gtk4::subclass::prelude::DerivedObjectProperties;
use std::cell::RefCell;

#[derive(glib::Properties, Default)]
#[properties(wrapper_type = super::PacketObject)]
pub struct PacketObject {
    #[property(construct, get, set)]
    pub packet_type: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for PacketObject {
    const NAME: &'static str = "PacketObject";
    type Type = super::PacketObject;
    type ParentType = glib::Object;
}

#[glib::derived_properties]
impl ObjectImpl for PacketObject {}

impl PacketObject {}

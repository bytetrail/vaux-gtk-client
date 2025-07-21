use glib::{
    object::ObjectExt,
    subclass::{object::ObjectImpl, types::ObjectSubclass},
};
use gtk4::subclass::prelude::DerivedObjectProperties;
use std::cell::{Cell, RefCell};

#[derive(glib::Properties, Default)]
#[properties(wrapper_type = super::Subscription)]
pub struct Subscription {
    #[property(construct, get, set)]
    pub topic: RefCell<String>,
    #[property(construct, get, set)]
    pub active: Cell<bool>,
    #[property(construct, get, set)]
    pub id: Cell<u32>,
}

#[glib::object_subclass]
impl ObjectSubclass for Subscription {
    const NAME: &'static str = "PacketObject";
    type Type = super::Subscription;
    type ParentType = glib::Object;
}

#[glib::derived_properties]
impl ObjectImpl for Subscription {}

impl Subscription {}

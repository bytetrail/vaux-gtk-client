use glib::Object;
use vaux_mqtt::PacketType;

mod objimpl;

glib::wrapper! {
    pub struct PacketObject(ObjectSubclass<objimpl::PacketObject>);
}

impl PacketObject {
    pub fn new(packet: vaux_mqtt::Packet) -> Self {
        Object::builder()
            .property("packet-type", PacketType::from(&packet).to_string())
            .build()
    }
}

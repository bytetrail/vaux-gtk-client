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
            .property("packet-id", PacketObject::packet_id_from(&packet))
            .build()
    }

    fn packet_id_from(packet: &vaux_mqtt::Packet) -> i32 {
        let id = match packet {
            vaux_mqtt::Packet::Connect(_) => None,
            vaux_mqtt::Packet::ConnAck(_) => None,
            vaux_mqtt::Packet::Publish(publish) => publish.packet_id,
            vaux_mqtt::Packet::PubAck(puback) => Some(puback.packet_id),
            vaux_mqtt::Packet::PubRec(pubrec) => Some(pubrec.packet_id),
            vaux_mqtt::Packet::PubRel(pubrel) => Some(pubrel.packet_id),
            vaux_mqtt::Packet::PubComp(pubcomp) => Some(pubcomp.packet_id),
            vaux_mqtt::Packet::Subscribe(subscribe) => Some(subscribe.packet_id()),
            vaux_mqtt::Packet::SubAck(suback) => Some(suback.packet_id()),
            vaux_mqtt::Packet::Unsubscribe(unsubscribe) => Some(unsubscribe.packet_id),
            vaux_mqtt::Packet::UnsubAck(unsuback) => Some(unsuback.packet_id()),
            vaux_mqtt::Packet::PingRequest(_) => None,
            vaux_mqtt::Packet::PingResponse(_) => None,
            vaux_mqtt::Packet::Disconnect(_) => None,
        };
        if let Some(id) = id { id as i32 } else { 0 }
    }
}

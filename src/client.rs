use std::{cell::RefCell, rc::Rc, time::Duration};
use tokio::{select, task::JoinHandle};
use vaux_client::{ClientBuilder, MqttConnection, client::ClientError, session::SessionState};
use vaux_mqtt::{QoSLevel, publish::Publish, unsubscribe};

pub const DEFAULT_WILL_DELAY_SECONDS: u32 = 60; // 1 minute
pub const DEFAULT_WILL_EXPIRY_SECONDS: u32 = 300; // 5 minutes

#[derive(Debug, Clone)]
pub struct ClientSetting {
    pub clean_start: Rc<RefCell<Option<bool>>>,
    pub client_id: Rc<RefCell<String>>,
    pub with_tls: Rc<RefCell<bool>>,
    pub ca_file: Rc<RefCell<String>>,
    pub client_cert: Rc<RefCell<String>>,
    pub host: Rc<RefCell<String>>,
    pub port: Rc<RefCell<u16>>,
    pub session_expiry: Rc<RefCell<u32>>,
    pub auto_ack: Rc<RefCell<bool>>,
    pub auto_packet_id: Rc<RefCell<bool>>,
    pub with_ping_resp: Rc<RefCell<bool>>,

    pub with_credentials: Rc<RefCell<bool>>,
    pub username: Rc<RefCell<String>>,
    pub password: Rc<RefCell<String>>,

    pub with_will: Rc<RefCell<bool>>,
    pub will_topic: Rc<RefCell<String>>,
    pub will_payload: Rc<RefCell<String>>,
    pub will_qos: Rc<RefCell<vaux_mqtt::QoSLevel>>,
    pub will_retain: Rc<RefCell<bool>>,
    pub will_delay: Rc<RefCell<u32>>,
    pub will_expiry: Rc<RefCell<u32>>,
}

impl ClientSetting {
    pub fn new() -> Self {
        Self {
            clean_start: Rc::new(RefCell::new(None)),
            client_id: Rc::new(RefCell::new(format!("client-{}", uuid::Uuid::new_v4()))),

            with_tls: Rc::new(RefCell::new(false)),
            ca_file: Rc::new(RefCell::new(String::new())),
            client_cert: Rc::new(RefCell::new(String::new())),

            host: Rc::new(RefCell::new("localhost".to_string())),
            port: Rc::new(RefCell::new(1883)),
            session_expiry: Rc::new(RefCell::new(DEFAULT_WILL_EXPIRY_SECONDS)),
            auto_ack: Rc::new(RefCell::new(true)),
            auto_packet_id: Rc::new(RefCell::new(true)),
            with_ping_resp: Rc::new(RefCell::new(true)),

            with_credentials: Rc::new(RefCell::new(false)),
            username: Rc::new(RefCell::new(String::new())),
            password: Rc::new(RefCell::new(String::new())),

            with_will: Rc::new(RefCell::new(false)),
            will_topic: Rc::new(RefCell::new(String::new())),
            will_payload: Rc::new(RefCell::new(String::new())),
            will_qos: Rc::new(RefCell::new(vaux_mqtt::QoSLevel::AtMostOnce)), // Default Qo
            will_retain: Rc::new(RefCell::new(false)),
            will_delay: Rc::new(RefCell::new(DEFAULT_WILL_DELAY_SECONDS)), // Default delay
            will_expiry: Rc::new(RefCell::new(DEFAULT_WILL_EXPIRY_SECONDS)),
        }
    }
}

pub enum Command {
    StartClient(ClientBuilder),
    ResumeSession(MqttConnection),
    Ping,
    Publish(Publish),                 // publish packet
    Subscribe(u16, QoSLevel, String), // topic
    Unsubscribe(u16, String),         // topic
    StopClient,
    #[allow(dead_code)]
    StopRunner,
}

pub async fn run(
    mqtt_tx: tokio::sync::mpsc::Sender<vaux_mqtt::Packet>,
    mut cmd_channel: tokio::sync::mpsc::Receiver<Command>,
) {
    let mut running = true;
    let mut client: Option<vaux_client::MqttClient> = None; // Placeholder for the client instance
    let (_dummy_tx, dummy_rx) = tokio::sync::mpsc::channel(1);
    let mut packet_consumer: tokio::sync::mpsc::Receiver<vaux_mqtt::Packet> = dummy_rx;

    let mut session: Option<vaux_client::session::SessionState> = None;
    let mut handle: Option<JoinHandle<Result<SessionState, ClientError>>> = None;

    while running {
        select! {
            packet = packet_consumer.recv() => {
                if let Some(p) = packet {
                        // Process the received packet
                        println!("Received packet: {p:?}");
                        match &p {
                            vaux_mqtt::Packet::UnsubAck(ack) => {
                                println!("UnsubAck received for packet ID: {}", ack.packet_id());
                                for (i, code) in ack.reason_code().iter().enumerate() {
                                    println!("  Reason code {}: {:?}", i + 1, code);
                                }
                            }
                            _ => {}
                        }
                        // Here you can handle the packet, e.g., update UI or store it
                        mqtt_tx.send(p).await.expect("Failed to send packet");
                }
            }
            command = cmd_channel.recv() => {
                match command {
                    Some(Command::ResumeSession(conn)) => {
                        // Logic to resume the session
                        println!("Resuming session");
                        let mut c = vaux_client::ClientBuilder::default()
                            .with_state(conn, session.take()
                            .expect("No session to resume"))
                            .build().await.expect("Failed to build client");
                        match c.try_start(Duration::from_secs(10), false).await {
                            Ok(h) => {
                                handle = Some(h);
                                // take the packet consumer
                                packet_consumer = c.take_packet_consumer().expect("Failed to take packet consumer");
                                client = Some(c);
                            }
                            Err(e) => {
                                eprintln!("Failed to connect MQTT Client: {e}");
                            }
                        }
                    }
                    Some(Command::StartClient(builder)) => {
                        // Logic to start the client
                        println!("MQTT Client started with builder");
                        match builder.build().await {
                            Ok(mut c) => {
                                println!("MQTT Client connected successfully");
                                // attempt to connect the client
                                match c.try_start(Duration::from_secs(10), true).await {
                                    Ok(h) => {
                                        handle = Some(h);
                                        // take the packet consumer
                                        packet_consumer = c.take_packet_consumer().expect("Failed to take packet consumer");
                                        client = Some(c);
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to connect MQTT Client: {e}");
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to start MQTT Client: {e}");
                            }
                        }
                    }
                    Some(Command::Ping) => {
                        if let Some(ref mut c) = client {
                            if let Err(e) = c.ping().await {
                                eprintln!("Failed to send ping: {e}");
                            }
                        } else {
                            eprintln!("Client not initialized, cannot send ping");
                        }
                    }
                    Some(Command::Publish(publish))=> {
                        let packet = vaux_mqtt::Packet::Publish(publish);
                        println!("Publishing packet: {packet:?}");
                        if let Some(ref mut c) = client {
                            c.packet_producer()
                            .send(packet)
                            .await
                            .expect("Failed to send publish packet");
                        } else {
                            eprintln!("Client not initialized, cannot publish");
                        }
                    }
                    Some(Command::Subscribe(packet_id, qos_level, topic)) => {
                        // Logic to subscribe to a topic
                        println!("Subscribed to topic '{topic}'" );
                        if let Some(ref mut c) = client {
                            let topic_list = vec![topic.as_str()];
                            match c.subscribe(packet_id, topic_list.as_slice(), qos_level).await {
                                Ok(_) => println!("Subscription request sent for topic '{topic}'"),
                                Err(e) => eprintln!("Failed to send subscription request: {e}"),
                            }
                        } else {
                            eprintln!("Client not initialized, cannot subscribe");
                        }
                    }
                    Some(Command::Unsubscribe(packet_id, topic)) => {
                        // Logic to unsubscribe from a topic
                        println!("Unsubscribed from topic '{topic}'");
                        let unsub = unsubscribe::Unsubscribe::new(packet_id, vec![topic]);
                        if let Some(ref mut c) = client {
                            c.packet_producer()
                            .send(vaux_mqtt::Packet::Unsubscribe(unsub))
                            .await
                            .expect("Failed to send unsubscribe packet");
                        } else {
                            eprintln!("Client not initialized, cannot unsubscribe");
                        }
                    }
                    Some(Command::StopClient) => {
                        // Logic to stop the client
                        println!("MQTT Client stopped");
                        // Reset the packet consumer to a dummy receiver
                        let (_dummy_tx, dummy_rx) = tokio::sync::mpsc::channel(1);
                        packet_consumer = dummy_rx; // Reset the packet consumer
                        if let Some(mut c) = client.take() {
                            if let Err(e) = c.stop().await {
                                eprintln!("Failed to stop MQTT Client: {e}");
                            } else {
                                println!("MQTT Client stopped successfully");
                            }
                        } else {
                            println!("No MQTT Client to stop");
                        }
                        // join the handle if it exists
                        if let Some(h) = handle.take() {
                            match h.await {
                                Ok(r) => {
                                     println!("Client session ended successfully");
                                     match r {
                                        Ok(s) => session = Some(s),
                                        Err(e) => eprintln!("Error in client session: {e}"),
                                     }
                                }
                                Err(e) => eprintln!("Error while ending client session: {e}"),
                            }
                        }
                    }
                    Some(Command::StopRunner) => {
                        // Logic to stop the runner
                        println!("Runner stopped");
                        running = false;
                    }
                    None => {
                        // Channel closed, exit loop
                        println!("Command channel closed, exiting");
                        running = false;
                    }
                }
            }
        }
    }
}

use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};
use tokio::{select, sync::Mutex};
use vaux_client::ClientBuilder;

pub const DEFAULT_EXPIRY_SECONDS: u32 = 300; // 5 minutes

#[derive(Debug, Clone)]
pub struct ClientSetting {
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
    pub will_expiry: Rc<RefCell<u32>>,
}

impl ClientSetting {
    pub fn new() -> Self {
        Self {
            client_id: Rc::new(RefCell::new(format!("client-{}", uuid::Uuid::new_v4()))),

            with_tls: Rc::new(RefCell::new(false)),
            ca_file: Rc::new(RefCell::new(String::new())),
            client_cert: Rc::new(RefCell::new(String::new())),

            host: Rc::new(RefCell::new("localhost".to_string())),
            port: Rc::new(RefCell::new(1883)),
            session_expiry: Rc::new(RefCell::new(DEFAULT_EXPIRY_SECONDS)),
            auto_ack: Rc::new(RefCell::new(true)),
            auto_packet_id: Rc::new(RefCell::new(true)),
            with_ping_resp: Rc::new(RefCell::new(false)),

            with_credentials: Rc::new(RefCell::new(false)),
            username: Rc::new(RefCell::new(String::new())),
            password: Rc::new(RefCell::new(String::new())),

            with_will: Rc::new(RefCell::new(true)),
            will_topic: Rc::new(RefCell::new(String::new())),
            will_payload: Rc::new(RefCell::new(String::new())),
            will_qos: Rc::new(RefCell::new(vaux_mqtt::QoSLevel::AtMostOnce)), // Default Qo
            will_retain: Rc::new(RefCell::new(false)),
            will_expiry: Rc::new(RefCell::new(DEFAULT_EXPIRY_SECONDS)),
        }
    }
}

pub enum Command {
    StartClient(ClientBuilder),
    Ping,
    Publish(String, String), // topic, payload
    Subscribe(String),       // topic
    Unsubscribe(String),     // topic
    StopClient,
    StopRunner,
}

pub async fn run(
    mqtt_tx: tokio::sync::mpsc::Sender<vaux_mqtt::Packet>,
    mut cmd_channel: tokio::sync::mpsc::Receiver<Command>,
) {
    let mut running = true;
    let mut client: Option<vaux_client::MqttClient> = None; // Placeholder for the client instance
    let mut packet_consumer: Option<tokio::sync::mpsc::Receiver<vaux_mqtt::Packet>> = None; // Placeholder for the packet consumer

    while running {
        select! {
            packet = packet_consumer.as_mut().unwrap().recv(), if packet_consumer.as_ref().is_some() => {
                match packet {
                    Some(p) => {
                        // Process the received packet
                        println!("Received packet: {:?}", p);
                        // Here you can handle the packet, e.g., update UI or store it
                        mqtt_tx.send(p).await.expect("Failed to send packet");
                    }
                    None => {
                        println!("Packet consumer channel closed");
                    }
                }
            }
            command = cmd_channel.recv() => {
                match command {
                    Some(Command::StartClient(builder)) => {
                        // Logic to start the client
                        println!("MQTT Client started with builder");
                        match builder.build().await {
                            Ok(mut c) => {
                                println!("MQTT Client connected successfully");
                                // attempt to connect the client
                                match c.try_start(Duration::from_secs(10), true).await {
                                    Ok(_) => {
                                        // take the packet consumer
                                        packet_consumer = c.take_packet_consumer();
                                        client = Some(c);
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to connect MQTT Client: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to start MQTT Client: {}", e);
                            }
                        }
                    }
                    Some(Command::Ping) => {
                        if let Some(ref mut c) = client {
                            if let Err(e) = c.ping().await {
                                eprintln!("Failed to send ping: {}", e);
                            }
                        } else {
                            eprintln!("Client not initialized, cannot send ping");
                        }
                    }
                    Some(Command::Publish(topic, payload)) => {
                        // Logic to publish a message
                        println!("Published to topic '{}': {}", topic, payload);
                    }
                    Some(Command::Subscribe(topic)) => {
                        // Logic to subscribe to a topic
                        println!("Subscribed to topic '{}'", topic);
                    }
                    Some(Command::Unsubscribe(topic)) => {
                        // Logic to unsubscribe from a topic
                        println!("Unsubscribed from topic '{}'", topic);
                    }
                    Some(Command::StopClient) => {
                        // Logic to stop the client
                        println!("MQTT Client stopped");
                        if let Some(mut c) = client.take() {
                            if let Err(e) = c.stop().await {
                                eprintln!("Failed to stop MQTT Client: {}", e);
                            } else {
                                println!("MQTT Client stopped successfully");
                            }
                        } else {
                            println!("No MQTT Client to stop");
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

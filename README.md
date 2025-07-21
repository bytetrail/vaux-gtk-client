Vaux GTK4 Test Client
========================

This is a test client for the Vaux MQTT client and server, built using GTK4 and Rust. It provides a graphical interface to interact with the MQTT client allowing users to publish and subscribe to topics. It is designed to be a simple and effective way to test the functionality of the Vaux MQTT client.

## Features
- Connect to an MQTT broker
    - WILL message support
    - Username and password authentication
    - TLS support for secure connections(1)
- Disconnect
- Publish messages to topics
- Subscribe to topics and receive messages
- View received messages in a user-friendly interface  


(1) mTLS is currently not supported, although there is a client certificate upload UI element. mTLS support will be added to vaux client in the future.


### Subscriptions
Subscriptions are managed through the 'Subscriptions' tab in the main window. Users can add, remove, and manage subscriptions to various topics. Each subscription can be configured with a topic, QoS level, and whether it is active or not. Each subscription supports a single topic filter. Future versions may support multiple topic filters per subscription.

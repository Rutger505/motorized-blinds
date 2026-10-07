//! Home Assistant integration over MQTT.

mod client;
pub mod packet;

use core::fmt::Write as _;

use embedded_io_async::{Read, Write};
use heapless::String;
use protocol::Command;

pub use client::{Error, MqttClient};
use packet::{Connect, Packet, Will};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_os = "none", derive(defmt::Format))]
pub enum CoverState {
    Open,
    Closed,
    Opening,
    Closing,
    Stopped,
}

impl CoverState {
    pub const fn as_str(self) -> &'static str {
        match self {
            CoverState::Open => "open",
            CoverState::Closed => "closed",
            CoverState::Opening => "opening",
            CoverState::Closing => "closing",
            CoverState::Stopped => "stopped",
        }
    }
}

/// `position` follows the Home Assistant convention: 100 is fully open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_os = "none", derive(defmt::Format))]
pub struct Status {
    pub state: CoverState,
    pub position: u8,
}

pub const KEEP_ALIVE_SECS: u16 = 60;
const BATTERY_TOPIC: &str = "afstandsbediening/batterij";
const ONLINE: &[u8] = b"online";
const OFFLINE: &[u8] = b"offline";

type Topic = String<48>;

struct Topics {
    set: Topic,
    state: Topic,
    position: Topic,
    availability: Topic,
}

impl Topics {
    fn new(id: u8) -> Self {
        let topic = |name: &str| {
            let mut topic = Topic::new();
            write!(topic, "rolgordijn/{id}/{name}").unwrap();
            topic
        };
        Self {
            set: topic("set"),
            state: topic("state"),
            position: topic("position"),
            availability: topic("availability"),
        }
    }
}

pub struct Credentials<'a> {
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
}

pub fn parse_command(payload: &[u8]) -> Option<Command> {
    match payload {
        b"OPEN" => Some(Command::Omhoog),
        b"CLOSE" => Some(Command::Omlaag),
        b"STOP" => Some(Command::Stop),
        b"AANGEPAST" => Some(Command::NaarAangepast),
        _ => None,
    }
}

pub struct HomeAssistant<T> {
    client: MqttClient<T>,
    id: u8,
    topics: Topics,
}

impl<T: Read + Write> HomeAssistant<T> {
    /// Connects, listens for commands and marks the blind as available.
    /// The broker marks it unavailable again when the connection drops.
    pub async fn connect(
        transport: T,
        id: u8,
        credentials: &Credentials<'_>,
    ) -> Result<Self, Error<T::Error>> {
        let topics = Topics::new(id);
        let mut client_id = String::<16>::new();
        write!(client_id, "rolgordijn-{id}").unwrap();

        let mut client = MqttClient::new(transport);
        client
            .connect(&Connect {
                client_id: &client_id,
                keep_alive_secs: KEEP_ALIVE_SECS,
                username: credentials.username,
                password: credentials.password,
                will: Some(Will {
                    topic: &topics.availability,
                    message: OFFLINE,
                }),
            })
            .await?;
        client.subscribe(&topics.set).await?;
        client.publish(&topics.availability, ONLINE, true).await?;

        Ok(Self { client, id, topics })
    }

    /// Publishes the MQTT discovery configs, so Home Assistant adds the
    /// blind, its custom position button and the remote's battery by itself.
    pub async fn announce(&mut self) -> Result<(), Error<T::Error>> {
        let id = self.id;
        let Topics {
            set,
            state,
            position,
            availability,
        } = &self.topics;
        let device = Device(id);

        let mut topic = String::<64>::new();
        let mut config = String::<768>::new();

        write!(topic, "homeassistant/cover/rolgordijn_{id}/config").unwrap();
        write!(
            config,
            r#"{{"name":null,"unique_id":"rolgordijn_{id}","device_class":"shade","#,
        )
        .unwrap();
        write!(
            config,
            r#""command_topic":"{set}","state_topic":"{state}","position_topic":"{position}","#,
        )
        .unwrap();
        write!(
            config,
            r#""availability_topic":"{availability}","payload_open":"OPEN","payload_close":"CLOSE","payload_stop":"STOP","position_open":100,"position_closed":0,{device}}}"#,
        )
        .unwrap();
        self.client.publish(&topic, config.as_bytes(), true).await?;

        topic.clear();
        config.clear();
        write!(
            topic,
            "homeassistant/button/rolgordijn_{id}_aangepast/config"
        )
        .unwrap();
        write!(
            config,
            r#"{{"name":"Aangepast","unique_id":"rolgordijn_{id}_aangepast","command_topic":"{set}","payload_press":"AANGEPAST","availability_topic":"{availability}",{device}}}"#,
        )
        .unwrap();
        self.client.publish(&topic, config.as_bytes(), true).await?;

        config.clear();
        write!(
            config,
            r#"{{"name":"Afstandsbediening batterij","unique_id":"afstandsbediening_batterij","device_class":"battery","unit_of_measurement":"%","state_topic":"{BATTERY_TOPIC}"}}"#,
        )
        .unwrap();
        self.client
            .publish(
                "homeassistant/sensor/afstandsbediening_batterij/config",
                config.as_bytes(),
                true,
            )
            .await
    }

    /// Waits for the next valid command on the blind's set topic.
    pub async fn next_command(&mut self) -> Result<Command, Error<T::Error>> {
        let set = &self.topics.set;
        self.client
            .receive(|packet| match packet {
                Packet::Publish { topic, payload } if topic == set.as_str() => {
                    parse_command(payload)
                }
                _ => None,
            })
            .await
    }

    pub async fn publish(&mut self, status: Status) -> Result<(), Error<T::Error>> {
        self.client
            .publish(&self.topics.state, status.state.as_str().as_bytes(), true)
            .await?;
        self.client
            .publish(
                &self.topics.position,
                number(status.position).as_bytes(),
                true,
            )
            .await
    }

    pub async fn publish_battery(&mut self, percent: u8) -> Result<(), Error<T::Error>> {
        self.client
            .publish(BATTERY_TOPIC, number(percent).as_bytes(), true)
            .await
    }

    pub async fn ping(&mut self) -> Result<(), Error<T::Error>> {
        self.client.ping().await
    }
}

struct Device(u8);

impl core::fmt::Display for Device {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let id = self.0;
        write!(
            f,
            r#""device":{{"identifiers":["rolgordijn_{id}"],"name":"Rolgordijn {id}","model":"ESP32-C6 + TMC2209"}}"#,
        )
    }
}

fn number(value: u8) -> String<3> {
    let mut text = String::new();
    write!(text, "{value}").unwrap();
    text
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec::Vec;

    use super::*;
    use embassy_futures::block_on;
    use embedded_io_async::ErrorType;

    /// Replays scripted broker bytes a few at a time, and records what the
    /// client sends.
    struct FakeBroker {
        incoming: Vec<u8>,
        read_at: usize,
        sent: Vec<u8>,
    }

    impl FakeBroker {
        fn replying(packets: &[&[u8]]) -> Self {
            Self {
                incoming: packets.concat(),
                read_at: 0,
                sent: Vec::new(),
            }
        }

        fn sent_packets(&self) -> Vec<Packet<'_>> {
            let mut packets = Vec::new();
            let mut rest = &self.sent[..];
            while let Some((packet, len)) = packet::decode(rest).unwrap() {
                packets.push(packet);
                rest = &rest[len..];
            }
            assert!(rest.is_empty());
            packets
        }

        fn sent_publishes(&self) -> Vec<(&str, &str)> {
            self.sent_packets()
                .into_iter()
                .filter_map(|packet| match packet {
                    Packet::Publish { topic, payload } => {
                        Some((topic, core::str::from_utf8(payload).unwrap()))
                    }
                    _ => None,
                })
                .collect()
        }
    }

    impl ErrorType for &mut FakeBroker {
        type Error = core::convert::Infallible;
    }

    impl Read for &mut FakeBroker {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            let end = (self.read_at + 3).min(self.incoming.len());
            let len = (end - self.read_at).min(buf.len());
            buf[..len].copy_from_slice(&self.incoming[self.read_at..self.read_at + len]);
            self.read_at += len;
            Ok(len)
        }
    }

    impl Write for &mut FakeBroker {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.sent.extend_from_slice(buf);
            Ok(buf.len())
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    const CONNACK_OK: &[u8] = &[0x20, 2, 0, 0];

    fn publish(topic: &str, payload: &[u8]) -> Vec<u8> {
        let mut out = [0; 256];
        let len = packet::publish(&mut out, topic, payload, false).unwrap();
        out[..len].to_vec()
    }

    const NO_CREDENTIALS: Credentials = Credentials {
        username: None,
        password: None,
    };

    fn connected(broker: &mut FakeBroker) -> HomeAssistant<&mut FakeBroker> {
        block_on(HomeAssistant::connect(broker, 2, &NO_CREDENTIALS)).unwrap()
    }

    #[test]
    fn parses_home_assistant_payloads() {
        assert_eq!(parse_command(b"OPEN"), Some(Command::Omhoog));
        assert_eq!(parse_command(b"CLOSE"), Some(Command::Omlaag));
        assert_eq!(parse_command(b"STOP"), Some(Command::Stop));
        assert_eq!(parse_command(b"AANGEPAST"), Some(Command::NaarAangepast));
        assert_eq!(parse_command(b"open"), None);
    }

    #[test]
    fn connect_subscribes_and_goes_online() {
        let mut broker = FakeBroker::replying(&[CONNACK_OK]);
        connected(&mut broker);

        let mut connect = [0; 128];
        let len = packet::connect(
            &mut connect,
            &Connect {
                client_id: "rolgordijn-2",
                keep_alive_secs: KEEP_ALIVE_SECS,
                username: None,
                password: None,
                will: Some(Will {
                    topic: "rolgordijn/2/availability",
                    message: b"offline",
                }),
            },
        )
        .unwrap();
        assert!(broker.sent.starts_with(&connect[..len]));

        let mut subscribe = [0; 64];
        let len = packet::subscribe(&mut subscribe, 1, "rolgordijn/2/set").unwrap();
        assert!(broker.sent[..].windows(len).any(|w| w == &subscribe[..len]));

        assert_eq!(
            broker.sent_publishes(),
            [("rolgordijn/2/availability", "online")]
        );
    }

    #[test]
    fn refused_connection_is_an_error() {
        let mut broker = FakeBroker::replying(&[&[0x20, 2, 0, 5]]);
        let result = block_on(HomeAssistant::connect(&mut broker, 1, &NO_CREDENTIALS));
        assert!(matches!(result, Err(Error::Refused(5))));
    }

    #[test]
    fn closed_connection_is_an_error() {
        let mut broker = FakeBroker::replying(&[CONNACK_OK]);
        let mut home_assistant = connected(&mut broker);
        assert!(matches!(
            block_on(home_assistant.next_command()),
            Err(Error::Closed)
        ));
    }

    #[test]
    fn next_command_skips_other_topics_and_unknown_payloads() {
        let mut broker = FakeBroker::replying(&[
            CONNACK_OK,
            &[0x90, 3, 0, 1, 0],
            &publish("rolgordijn/1/set", b"OPEN"),
            &publish("rolgordijn/2/set", b"DANCE"),
            &[0xD0, 0],
            &publish("rolgordijn/2/set", b"CLOSE"),
            &publish("rolgordijn/2/set", b"AANGEPAST"),
        ]);
        let mut home_assistant = connected(&mut broker);

        assert_eq!(
            block_on(home_assistant.next_command()).unwrap(),
            Command::Omlaag
        );
        assert_eq!(
            block_on(home_assistant.next_command()).unwrap(),
            Command::NaarAangepast
        );
    }

    #[test]
    fn publishes_status_and_battery_retained() {
        let mut broker = FakeBroker::replying(&[CONNACK_OK]);
        let mut home_assistant = connected(&mut broker);
        block_on(home_assistant.publish(Status {
            state: CoverState::Closing,
            position: 42,
        }))
        .unwrap();
        block_on(home_assistant.publish_battery(100)).unwrap();

        assert_eq!(
            &broker.sent_publishes()[1..],
            [
                ("rolgordijn/2/state", "closing"),
                ("rolgordijn/2/position", "42"),
                ("afstandsbediening/batterij", "100"),
            ]
        );
    }

    #[test]
    fn announce_publishes_valid_discovery_configs() {
        let mut broker = FakeBroker::replying(&[CONNACK_OK]);
        let mut home_assistant = connected(&mut broker);
        block_on(home_assistant.announce()).unwrap();

        let publishes = broker.sent_publishes();
        let configs: Vec<_> = publishes[1..]
            .iter()
            .map(|(topic, payload)| {
                let json: serde_json::Value = serde_json::from_str(payload).unwrap();
                (*topic, json)
            })
            .collect();

        let topics: Vec<_> = configs.iter().map(|(topic, _)| *topic).collect();
        assert_eq!(
            topics,
            [
                "homeassistant/cover/rolgordijn_2/config",
                "homeassistant/button/rolgordijn_2_aangepast/config",
                "homeassistant/sensor/afstandsbediening_batterij/config",
            ]
        );

        let cover = &configs[0].1;
        assert_eq!(cover["command_topic"], "rolgordijn/2/set");
        assert_eq!(cover["state_topic"], "rolgordijn/2/state");
        assert_eq!(cover["position_topic"], "rolgordijn/2/position");
        assert_eq!(cover["availability_topic"], "rolgordijn/2/availability");
        assert_eq!(cover["device"]["identifiers"][0], "rolgordijn_2");

        let button = &configs[1].1;
        assert_eq!(button["payload_press"], "AANGEPAST");
        assert_eq!(button["command_topic"], "rolgordijn/2/set");

        let battery = &configs[2].1;
        assert_eq!(battery["state_topic"], "afstandsbediening/batterij");
        assert_eq!(battery["unit_of_measurement"], "%");
    }
}

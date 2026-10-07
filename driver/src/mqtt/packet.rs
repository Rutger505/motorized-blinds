//! Encoding and decoding of the MQTT 3.1.1 packets this client needs, at QoS 0.

const CONNECT: u8 = 0x10;
const CONNACK: u8 = 0x20;
const PUBLISH: u8 = 0x30;
const SUBSCRIBE: u8 = 0x82;
const SUBACK: u8 = 0x90;
const PINGREQ: u8 = 0xC0;
const PINGRESP: u8 = 0xD0;

const RETAIN: u8 = 0x01;
const QOS_MASK: u8 = 0x06;

const CLEAN_SESSION: u8 = 0x02;
const WILL: u8 = 0x04;
const WILL_RETAIN: u8 = 0x20;
const PASSWORD: u8 = 0x40;
const USERNAME: u8 = 0x80;

/// The fixed header is at most 1 type byte and 4 length bytes.
const MAX_FIXED_HEADER: usize = 5;
const MAX_REMAINING_LENGTH: usize = 268_435_455;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_os = "none", derive(defmt::Format))]
pub struct BufferFull;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_os = "none", derive(defmt::Format))]
pub struct Malformed;

pub struct Will<'a> {
    pub topic: &'a str,
    pub message: &'a [u8],
}

pub struct Connect<'a> {
    pub client_id: &'a str,
    pub keep_alive_secs: u16,
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub will: Option<Will<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Packet<'a> {
    ConnAck { return_code: u8 },
    Publish { topic: &'a str, payload: &'a [u8] },
    SubAck,
    PingResp,
    Other,
}

struct Encoder<'a> {
    buffer: &'a mut [u8],
    len: usize,
}

impl Encoder<'_> {
    fn bytes(&mut self, bytes: &[u8]) -> Result<(), BufferFull> {
        let end = self.len + bytes.len();
        self.buffer
            .get_mut(self.len..end)
            .ok_or(BufferFull)?
            .copy_from_slice(bytes);
        self.len = end;
        Ok(())
    }

    fn byte(&mut self, byte: u8) -> Result<(), BufferFull> {
        self.bytes(&[byte])
    }

    fn u16(&mut self, value: u16) -> Result<(), BufferFull> {
        self.bytes(&value.to_be_bytes())
    }

    fn prefixed(&mut self, bytes: &[u8]) -> Result<(), BufferFull> {
        let len = u16::try_from(bytes.len()).map_err(|_| BufferFull)?;
        self.u16(len)?;
        self.bytes(bytes)
    }
}

/// Writes the body with `body`, then puts the fixed header in front of it.
fn encode(
    out: &mut [u8],
    packet_type: u8,
    body: impl FnOnce(&mut Encoder) -> Result<(), BufferFull>,
) -> Result<usize, BufferFull> {
    let mut encoder = Encoder {
        buffer: out.get_mut(MAX_FIXED_HEADER..).ok_or(BufferFull)?,
        len: 0,
    };
    body(&mut encoder)?;
    let body_len = encoder.len;

    let mut header = [packet_type, 0, 0, 0, 0];
    let header_len = 1 + encode_remaining_length(body_len, &mut header[1..]);
    out.copy_within(MAX_FIXED_HEADER..MAX_FIXED_HEADER + body_len, header_len);
    out[..header_len].copy_from_slice(&header[..header_len]);
    Ok(header_len + body_len)
}

fn encode_remaining_length(mut len: usize, out: &mut [u8]) -> usize {
    let mut written = 0;
    loop {
        let mut byte = (len % 128) as u8;
        len /= 128;
        if len > 0 {
            byte |= 0x80;
        }
        out[written] = byte;
        written += 1;
        if len == 0 {
            return written;
        }
    }
}

pub fn connect(out: &mut [u8], connect: &Connect) -> Result<usize, BufferFull> {
    let mut flags = CLEAN_SESSION;
    if connect.username.is_some() {
        flags |= USERNAME;
    }
    if connect.password.is_some() {
        flags |= PASSWORD;
    }
    if connect.will.is_some() {
        flags |= WILL | WILL_RETAIN;
    }

    encode(out, CONNECT, |encoder| {
        encoder.prefixed(b"MQTT")?;
        encoder.byte(4)?;
        encoder.byte(flags)?;
        encoder.u16(connect.keep_alive_secs)?;
        encoder.prefixed(connect.client_id.as_bytes())?;
        if let Some(will) = &connect.will {
            encoder.prefixed(will.topic.as_bytes())?;
            encoder.prefixed(will.message)?;
        }
        if let Some(username) = connect.username {
            encoder.prefixed(username.as_bytes())?;
        }
        if let Some(password) = connect.password {
            encoder.prefixed(password.as_bytes())?;
        }
        Ok(())
    })
}

pub fn publish(
    out: &mut [u8],
    topic: &str,
    payload: &[u8],
    retain: bool,
) -> Result<usize, BufferFull> {
    let packet_type = if retain { PUBLISH | RETAIN } else { PUBLISH };
    encode(out, packet_type, |encoder| {
        encoder.prefixed(topic.as_bytes())?;
        encoder.bytes(payload)
    })
}

pub fn subscribe(out: &mut [u8], packet_id: u16, topic: &str) -> Result<usize, BufferFull> {
    encode(out, SUBSCRIBE, |encoder| {
        encoder.u16(packet_id)?;
        encoder.prefixed(topic.as_bytes())?;
        encoder.byte(0)
    })
}

pub fn ping(out: &mut [u8]) -> Result<usize, BufferFull> {
    encode(out, PINGREQ, |_| Ok(()))
}

/// Decodes the first packet in `buffer`, and how many bytes it took.
/// `Ok(None)` means the packet isn't complete yet.
pub fn decode(buffer: &[u8]) -> Result<Option<(Packet<'_>, usize)>, Malformed> {
    let Some(&first) = buffer.first() else {
        return Ok(None);
    };

    let mut remaining_length = 0usize;
    let mut header_len = 1;
    loop {
        let Some(&byte) = buffer.get(header_len) else {
            return Ok(None);
        };
        remaining_length += usize::from(byte & 0x7F) << (7 * (header_len - 1));
        header_len += 1;
        if byte & 0x80 == 0 {
            break;
        }
        if header_len == MAX_FIXED_HEADER {
            return Err(Malformed);
        }
    }
    if remaining_length > MAX_REMAINING_LENGTH {
        return Err(Malformed);
    }

    let total = header_len + remaining_length;
    let Some(body) = buffer.get(header_len..total) else {
        return Ok(None);
    };

    let packet = match first & 0xF0 {
        CONNACK => Packet::ConnAck {
            return_code: *body.get(1).ok_or(Malformed)?,
        },
        PUBLISH => decode_publish(first, body)?,
        SUBACK => Packet::SubAck,
        PINGRESP => Packet::PingResp,
        _ => Packet::Other,
    };
    Ok(Some((packet, total)))
}

fn decode_publish(first: u8, body: &[u8]) -> Result<Packet<'_>, Malformed> {
    let [high, low, rest @ ..] = body else {
        return Err(Malformed);
    };
    let topic_len = usize::from(u16::from_be_bytes([*high, *low]));
    let topic = rest.get(..topic_len).ok_or(Malformed)?;
    let topic = core::str::from_utf8(topic).map_err(|_| Malformed)?;

    let packet_id_len = if first & QOS_MASK == 0 { 0 } else { 2 };
    let payload = rest.get(topic_len + packet_id_len..).ok_or(Malformed)?;
    Ok(Packet::Publish { topic, payload })
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    fn encoded(f: impl FnOnce(&mut [u8]) -> Result<usize, BufferFull>) -> std::vec::Vec<u8> {
        let mut out = [0; 512];
        let len = f(&mut out).unwrap();
        out[..len].to_vec()
    }

    #[test]
    fn encodes_connect_with_credentials_and_will() {
        let bytes = encoded(|out| {
            connect(
                out,
                &Connect {
                    client_id: "id",
                    keep_alive_secs: 60,
                    username: Some("u"),
                    password: Some("p"),
                    will: Some(Will {
                        topic: "t",
                        message: b"off",
                    }),
                },
            )
        });
        assert_eq!(
            bytes,
            [
                0x10, 28, 0, 4, b'M', b'Q', b'T', b'T', 4, 0xE6, 0, 60, 0, 2, b'i', b'd', 0, 1,
                b't', 0, 3, b'o', b'f', b'f', 0, 1, b'u', 0, 1, b'p'
            ]
        );
    }

    #[test]
    fn encodes_a_minimal_connect() {
        let bytes = encoded(|out| {
            connect(
                out,
                &Connect {
                    client_id: "x",
                    keep_alive_secs: 30,
                    username: None,
                    password: None,
                    will: None,
                },
            )
        });
        assert_eq!(
            bytes,
            [
                0x10, 13, 0, 4, b'M', b'Q', b'T', b'T', 4, 0x02, 0, 30, 0, 1, b'x'
            ]
        );
    }

    #[test]
    fn encodes_publish_and_subscribe() {
        assert_eq!(
            encoded(|out| publish(out, "a/b", b"on", true)),
            [0x31, 7, 0, 3, b'a', b'/', b'b', b'o', b'n']
        );
        assert_eq!(
            encoded(|out| subscribe(out, 1, "a/b")),
            [0x82, 8, 0, 1, 0, 3, b'a', b'/', b'b', 0]
        );
        assert_eq!(encoded(ping), [0xC0, 0]);
    }

    #[test]
    fn encodes_a_multi_byte_remaining_length() {
        let payload = [b'x'; 300];
        let bytes = encoded(|out| publish(out, "t", &payload, false));
        assert_eq!(&bytes[..3], &[0x30, 0xAF, 0x02]);
        assert_eq!(bytes.len(), 3 + 303);
        assert_eq!(
            decode(&bytes),
            Ok(Some((
                Packet::Publish {
                    topic: "t",
                    payload: &payload,
                },
                bytes.len()
            )))
        );
    }

    #[test]
    fn reports_a_full_buffer() {
        let mut out = [0; 8];
        assert_eq!(
            publish(&mut out, "topic", b"payload", false),
            Err(BufferFull)
        );
    }

    #[test]
    fn decodes_incoming_packets() {
        assert_eq!(
            decode(&[0x20, 2, 0, 5]),
            Ok(Some((Packet::ConnAck { return_code: 5 }, 4)))
        );
        assert_eq!(decode(&[0x90, 3, 0, 1, 0]), Ok(Some((Packet::SubAck, 5))));
        assert_eq!(decode(&[0xD0, 0, 0xFF]), Ok(Some((Packet::PingResp, 2))));
        assert_eq!(
            decode(&[0x30, 5, 0, 1, b't', b'h', b'i']),
            Ok(Some((
                Packet::Publish {
                    topic: "t",
                    payload: b"hi"
                },
                7
            )))
        );
    }

    #[test]
    fn skips_the_packet_id_of_a_qos_1_publish() {
        assert_eq!(
            decode(&[0x32, 7, 0, 1, b't', 0, 9, b'h', b'i']),
            Ok(Some((
                Packet::Publish {
                    topic: "t",
                    payload: b"hi"
                },
                9
            )))
        );
    }

    #[test]
    fn waits_for_an_incomplete_packet() {
        assert_eq!(decode(&[]), Ok(None));
        assert_eq!(decode(&[0x30]), Ok(None));
        assert_eq!(decode(&[0x30, 0x80]), Ok(None));
        assert_eq!(decode(&[0x30, 5, 0, 1, b't']), Ok(None));
    }

    #[test]
    fn rejects_malformed_packets() {
        assert_eq!(
            decode(&[0x30, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]),
            Err(Malformed)
        );
        assert_eq!(decode(&[0x30, 2, 0, 5]), Err(Malformed));
        assert_eq!(decode(&[0x30, 3, 0, 1, 0xFF]), Err(Malformed));
    }
}

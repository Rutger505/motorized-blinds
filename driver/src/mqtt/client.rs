use embedded_io_async::{Read, Write};

use super::packet::{self, BufferFull, Connect, Malformed, Packet};

const BUFFER_LEN: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(target_os = "none", derive(defmt::Format))]
pub enum Error<E> {
    Transport(E),
    Closed,
    BufferFull,
    Malformed,
    Refused(u8),
}

impl<E> From<BufferFull> for Error<E> {
    fn from(_: BufferFull) -> Self {
        Error::BufferFull
    }
}

impl<E> From<Malformed> for Error<E> {
    fn from(_: Malformed) -> Self {
        Error::Malformed
    }
}

/// Received bytes are kept in the client between calls, so `receive` can be
/// cancelled (e.g. by a `select`) without losing part of a packet.
pub struct MqttClient<T> {
    transport: T,
    rx: [u8; BUFFER_LEN],
    rx_len: usize,
    tx: [u8; BUFFER_LEN],
    next_packet_id: u16,
}

impl<T: Read + Write> MqttClient<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            rx: [0; BUFFER_LEN],
            rx_len: 0,
            tx: [0; BUFFER_LEN],
            next_packet_id: 1,
        }
    }

    pub async fn connect(&mut self, options: &Connect<'_>) -> Result<(), Error<T::Error>> {
        let len = packet::connect(&mut self.tx, options)?;
        self.send(len).await?;

        let return_code = self
            .receive(|packet| match packet {
                Packet::ConnAck { return_code } => Some(return_code),
                _ => None,
            })
            .await?;
        match return_code {
            0 => Ok(()),
            code => Err(Error::Refused(code)),
        }
    }

    pub async fn publish(
        &mut self,
        topic: &str,
        payload: &[u8],
        retain: bool,
    ) -> Result<(), Error<T::Error>> {
        let len = packet::publish(&mut self.tx, topic, payload, retain)?;
        self.send(len).await
    }

    pub async fn subscribe(&mut self, topic: &str) -> Result<(), Error<T::Error>> {
        let packet_id = self.next_packet_id;
        self.next_packet_id = self.next_packet_id.checked_add(1).unwrap_or(1);
        let len = packet::subscribe(&mut self.tx, packet_id, topic)?;
        self.send(len).await
    }

    pub async fn ping(&mut self) -> Result<(), Error<T::Error>> {
        let len = packet::ping(&mut self.tx)?;
        self.send(len).await
    }

    /// Reads packets until `select` picks one, and returns what it made of it.
    pub async fn receive<R>(
        &mut self,
        mut select: impl FnMut(Packet<'_>) -> Option<R>,
    ) -> Result<R, Error<T::Error>> {
        loop {
            while let Some((packet, len)) = packet::decode(&self.rx[..self.rx_len])? {
                let selected = select(packet);
                self.rx.copy_within(len..self.rx_len, 0);
                self.rx_len -= len;
                if let Some(selected) = selected {
                    return Ok(selected);
                }
            }

            if self.rx_len == BUFFER_LEN {
                return Err(Error::BufferFull);
            }
            let read = self
                .transport
                .read(&mut self.rx[self.rx_len..])
                .await
                .map_err(Error::Transport)?;
            if read == 0 {
                return Err(Error::Closed);
            }
            self.rx_len += read;
        }
    }

    async fn send(&mut self, len: usize) -> Result<(), Error<T::Error>> {
        self.transport
            .write_all(&self.tx[..len])
            .await
            .map_err(Error::Transport)?;
        self.transport.flush().await.map_err(Error::Transport)
    }
}

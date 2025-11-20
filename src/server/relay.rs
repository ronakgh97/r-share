use crate::config::{ACK_SIGNAL, BUFFER_SIZE, MAX_DONE_WAIT_MILLIS, READY_SIGNAL};
use crate::utils::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, ReadHalf, WriteHalf};
use tokio::net::{TcpSocket, TcpStream};

/// Transfer role in the relay session
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferRole {
    Sender,
    Receiver,
}

impl TransferRole {
    pub fn as_str(&self) -> &str {
        match self {
            TransferRole::Sender => "sender",
            TransferRole::Receiver => "receiver",
        }
    }
}

/// HTTP API request/response structures
#[derive(Debug, Serialize)]
struct ServeRequest {
    #[serde(rename = "senderFp")]
    sender_fingerprint: String,
    #[serde(rename = "receiverFp")]
    receiver_fingerprint: String,
    filename: String,
    #[serde(rename = "fileSize")]
    file_size: u64,
    signature: String,
    #[serde(rename = "fileHash")]
    file_hash: String,
    #[serde(rename = "senderEphemeralKey")]
    sender_ephemeral_key: String,
}

#[derive(Debug, Serialize)]
struct ListenRequest {
    #[serde(rename = "receiverFp")]
    receiver_fingerprint: String,
    #[serde(rename = "receiverEphemeralKey")]
    receiver_ephemeral_key: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ServeResponse {
    status: String,
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "socketPort")]
    socket_port: u16,
    message: String,
    #[serde(rename = "receiverEphemeralKey")]
    receiver_ephemeral_key: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ListenResponse {
    status: String,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    #[serde(rename = "senderFp")]
    sender_fp: Option<String>,
    filename: Option<String>,
    #[serde(rename = "fileSize")]
    file_size: Option<u64>,
    signature: Option<String>,
    #[serde(rename = "fileHash")]
    file_hash: Option<String>,
    #[serde(rename = "socketPort")]
    socket_port: Option<u16>,
    message: String,
    #[serde(rename = "senderEphemeralKey")]
    sender_ephemeral_key: Option<String>,
    #[serde(rename = "receiverEphemeralKey")]
    receiver_ephemeral_key: Option<String>,
}

/// Active transfer session with socket connection
#[allow(dead_code)]
pub struct TransferSession {
    session_id: String,
    role: TransferRole,
    buf_reader: BufReader<ReadHalf<TcpStream>>,
    buf_writer: BufWriter<WriteHalf<TcpStream>>,
    // Metadata (only populated for receiver)
    pub filename: Option<String>,
    pub file_size: Option<u64>,
    pub signature: Option<String>,
    pub sender_fp: Option<String>,
    pub file_hash: Option<String>,
    pub sender_ephemeral_key: Option<String>,
    pub receiver_ephemeral_key: Option<String>,
}

impl TransferSession {
    /// Read data from the socket connection
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.buf_reader
            .read(buf)
            .await
            .map_err(|_e| Error::NetworkError(format!("Failed to read from socket")))
    }

    /// Read exact amount of data from the socket connection
    pub async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.buf_reader.read_exact(buf).await?;
        Ok(())
    }

    /// Write data to the socket connection
    #[allow(dead_code)]
    pub async fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.buf_writer
            .write(data)
            .await
            .map_err(|_e| Error::NetworkError(format!("Failed to write to socket")))
    }

    /// Write all data to the socket connection
    pub async fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.buf_writer
            .write_all(data)
            .await
            .map_err(|_e| Error::NetworkError(format!("Failed to write all to socket")))
    }

    /// Flush the socket connection
    pub async fn flush(&mut self) -> Result<()> {
        self.buf_writer
            .flush()
            .await
            .map_err(|_e| Error::NetworkError(format!("Failed to flush socket")))
    }

    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the transfer role
    #[allow(dead_code)]
    pub fn role(&self) -> TransferRole {
        self.role
    }
}

/// Client for interacting with the relay server
pub struct RelayClient {
    server_ip: String,
    http_port: u16,
    socket_port: u16,
}

impl RelayClient {
    /// Create a new relay client
    pub fn new(server_ip: String, http_port: u16, socket_port: u16) -> Self {
        Self {
            server_ip,
            http_port,
            socket_port,
        }
    }

    pub async fn health_check(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!(
            "http://{}:{}/actuator/health",
            self.server_ip, self.http_port
        );

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|_e| Error::NetworkError(format!("Failed to call health API")))?;

        if !response.status().is_success() {
            return Err(Error::NetworkError(format!(
                "Health API failed with status :{}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Initiate a file transfer as sender (blocks until receiver connects)
    pub async fn serve(
        &self,
        sender_fingerprint: String,
        receiver_fingerprint: String,
        filename: String,
        file_size: u64,
        signature: String,
        file_hash: String,
        sender_ephemeral_key: String,
    ) -> Result<TransferSession> {
        // Call HTTP API to create session
        let client = reqwest::Client::new();
        let url = format!(
            "http://{}:{}/api/relay/serve",
            self.server_ip, self.http_port
        ); // TODO: USE HTTPS

        let request = ServeRequest {
            sender_fingerprint,
            receiver_fingerprint,
            filename,
            file_size,
            signature,
            file_hash,
            sender_ephemeral_key,
        };

        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|_e| Error::NetworkError(format!("Failed to call serve API")))?;

        if !response.status().is_success() {
            let status = response.status();
            //let _body = response.text().await.unwrap_or_default();
            return Err(Error::NetworkError(format!(
                "Connection timeout or refused, Status: {}",
                status
            )));
        }

        let session: ServeResponse = response
            .json()
            .await
            .map_err(|_e| Error::SessionError(format!("Failed to parse session response")))?;

        // Connect to socket server
        let socket = self
            .connect_socket(&session.session_id, TransferRole::Sender)
            .await?;

        let (read_half, write_half) = tokio::io::split(socket);
        let buf_reader = BufReader::with_capacity(BUFFER_SIZE, read_half);
        let buf_writer = BufWriter::with_capacity(BUFFER_SIZE, write_half);

        Ok(TransferSession {
            session_id: session.session_id,
            role: TransferRole::Sender,
            buf_reader,
            buf_writer,
            filename: None,
            file_size: None,
            signature: None,
            sender_fp: None,
            file_hash: None,
            sender_ephemeral_key: None,
            receiver_ephemeral_key: session.receiver_ephemeral_key,
        })
    }

    /// Join a file transfer as receiver (blocks until sender connects)
    pub async fn listen(
        &self,
        receiver_fingerprint: String,
        receiver_ephemeral_key: String,
    ) -> Result<TransferSession> {
        // Call HTTP API to join session
        let client = reqwest::Client::new();
        let url = format!(
            "http://{}:{}/api/relay/listen",
            self.server_ip, self.http_port
        ); // TODO: USE HTTPS

        let request = ListenRequest {
            receiver_fingerprint,
            receiver_ephemeral_key,
        };

        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|_e| Error::NetworkError(format!("Failed to call listen API")))?;

        if !response.status().is_success() {
            let status = response.status();
            //let _body = response.text().await.unwrap_or_default();
            return Err(Error::NetworkError(format!(
                "Connection timeout or refused, Status: {}",
                status
            )));
        }

        let session: ListenResponse = response
            .json()
            .await
            .map_err(|_e| Error::NetworkError(format!("Failed to parse session response")))?;

        // Extract required fields from response
        let session_id = session
            .session_id
            .ok_or_else(|| Error::SessionError("Server did not return session_id".into()))?;
        let filename = session
            .filename
            .ok_or_else(|| Error::SessionError("Server did not return filename".into()))?;
        let file_size = session
            .file_size
            .ok_or_else(|| Error::SessionError("Server did not return file_size".into()))?;
        let signature = session
            .signature
            .ok_or_else(|| Error::SessionError("Server did not return signature".into()))?;
        let sender_fp = session
            .sender_fp
            .ok_or_else(|| Error::SessionError("Server did not return sender_fp".into()))?;
        let file_hash = session
            .file_hash
            .ok_or_else(|| Error::SessionError("Server did not return file_hash".into()))?;
        let sender_ephemeral_key = session.sender_ephemeral_key.ok_or_else(|| {
            Error::NetworkError("Server did not return sender ephemeral key".into())
        })?;
        let receiver_ephemeral_key = session.receiver_ephemeral_key.ok_or_else(|| {
            Error::NetworkError("Server did not return receiver ephemeral key".into())
        })?;

        // Connect to socket server
        let socket = self
            .connect_socket(&session_id, TransferRole::Receiver)
            .await?;

        let (read_half, write_half) = tokio::io::split(socket);
        let buf_reader = BufReader::with_capacity(BUFFER_SIZE, read_half);
        let buf_writer = BufWriter::with_capacity(BUFFER_SIZE, write_half);

        Ok(TransferSession {
            session_id,
            role: TransferRole::Receiver,
            buf_reader,
            buf_writer,
            filename: Some(filename),
            file_size: Some(file_size),
            signature: Some(signature),
            sender_fp: Some(sender_fp),
            file_hash: Some(file_hash),
            sender_ephemeral_key: Some(sender_ephemeral_key),
            receiver_ephemeral_key: Some(receiver_ephemeral_key),
        })
    }

    /// Connect to the socket server and perform handshake
    async fn connect_socket(&self, session_id: &str, role: TransferRole) -> Result<TcpStream> {
        let addr_str = format!("{}:{}", self.server_ip, self.socket_port);
        let addr: SocketAddr = addr_str
            .parse()
            .map_err(|_e| Error::NetworkError(format!("Invalid socket address: {}", addr_str)))?;

        let socket = TcpSocket::new_v4().map_err(|_e| {
            Error::NetworkError(format!("Failed to connect to socket server: {}", addr_str))
        })?;

        socket
            .set_nodelay(true)
            .map_err(|_e| Error::NetworkError(format!("Failed to set TCP_NODELAY")))?;

        socket
            .set_send_buffer_size(BUFFER_SIZE as u32)
            .map_err(|_e| Error::NetworkError(format!("Failed to set send buffer")))?;
        socket
            .set_recv_buffer_size(BUFFER_SIZE as u32)
            .map_err(|_e| Error::NetworkError(format!("Failed to set recv buffer")))?;

        let mut socket = socket
            .connect(addr)
            .await
            .map_err(|_e| Error::NetworkError(format!("Failed to connect to socket server")))?;

        // Send handshake: "session_id:role"
        let handshake = format!("{}:{}\n", session_id, role.as_str());
        socket
            .write_all(handshake.as_bytes())
            .await
            .map_err(|_e| Error::NetworkError(format!("Failed to send handshake")))?;

        // Wait for READY signal from server (indicates pairing complete)
        let mut ready_buffer = [0u8; 6]; // "READY\n" is 6 bytes
        socket
            .read_exact(&mut ready_buffer)
            .await
            .map_err(|_e| Error::NetworkError(format!("Failed to read READY signal")))?;

        let ready_signal = String::from_utf8_lossy(&ready_buffer);
        if ready_signal.as_bytes() != READY_SIGNAL {
            return Err(Error::NetworkError(format!(
                "Expected READY signal, got: {}",
                ready_signal.trim()
            )));
        }

        // Send ACK to confirm we're ready to receive/send data
        socket
            .write_all(ACK_SIGNAL)
            .await
            .map_err(|_e| Error::NetworkError(format!("Failed to send ACK")))?;

        // VERY CRITICAL!!! -> Give server time to process ACK and activate relay before data starts flowing
        tokio::time::sleep(tokio::time::Duration::from_millis(MAX_DONE_WAIT_MILLIS)).await;

        Ok(socket)
    }
}

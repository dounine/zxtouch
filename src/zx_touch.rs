use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc};
use futures::lock::Mutex;
use tracing::{debug, error};
use crate::error::Error;
use crate::r#type::MessageType;


pub struct ZxTouch {
    host: String,
    port: i32,
    stream: Option<Arc<Mutex<TcpStream>>>,
}

impl ZxTouch {
    pub fn new<S: AsRef<str>, P: Into<i32>>(host: S, port: P) -> Self {
        Self {
            host: host.as_ref().to_string(),
            port: port.into(),
            stream: None,
        }
    }
    pub async fn close(&mut self) -> Result<(), Error> {
        match self.stream.take() {
            None => Ok(()),
            Some(mut socket) => {
                let mut socket = socket.lock().await;
                socket
                    .shutdown(std::net::Shutdown::Both)
                    .map_err(|e| Error::SocketError(e))
            }
        }
    }
    pub async fn connect(&mut self) -> Result<(), Error> {
        let addr: SocketAddr = format!("{}:{}", self.host, self.port).parse().unwrap();
        let stream = TcpStream::connect(addr).unwrap();
        self.stream = Some(Arc::new(Mutex::new(stream)));
        Ok(())
    }
    pub(crate) async fn touch(&self, touch_type: TouchType, x: u32, y: u32, finger: TouchFinger) -> Result<(), Error> {
        if let Some(mut socket) = self.stream.as_ref() {
            let message_type: u8 = MessageType::PerformTouch.into();
            let touch_type: u8 = touch_type.into();
            let finger: u8 = finger.into();
            let msg = format!("{};;1{}{}{:05}{:05}\r\n", message_type, touch_type, finger, x * 10, y * 10);
            let mut socket = socket.lock().await;
            match socket
                .write_all(msg.as_bytes()) {
                Ok(_) => {
                    debug!("send message: {}", msg);
                    Ok(())
                }
                Err(e) => {
                    error!("write error: {}", e);
                    Err(Error::SocketError(e))
                }
            }
        } else {
            Err(Error::SocketError(std::io::Error::new(std::io::ErrorKind::NotConnected, "not connected")))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TouchFinger {
    One,
    Two,
    Three,
    Four,
    Five,
}

pub(crate) enum TouchType {
    Down,
    Move,
    Up,
}

unsafe impl Send for TouchFinger {}

unsafe impl Sync for TouchFinger {}

impl Into<u8> for TouchType {
    fn into(self) -> u8 {
        match self {
            TouchType::Down => 1,
            TouchType::Move => 2,
            TouchType::Up => 0,
        }
    }
}

impl Into<u8> for TouchFinger {
    fn into(self) -> u8 {
        match self {
            TouchFinger::One => 1,
            TouchFinger::Two => 2,
            TouchFinger::Three => 3,
            TouchFinger::Four => 4,
            TouchFinger::Five => 5,
        }
    }
}

#[async_trait::async_trait]
pub trait TouchTrait {
    async fn show_alert_box(&self, title: &str, content: &str, duration: u32) -> Result<String, Error>;
    async fn touch_down(&self, x: u32, y: u32, finger: TouchFinger) -> Result<(), Error>;
    async fn touch_move(&self, x: u32, y: u32, finger: TouchFinger) -> Result<(), Error>;
    async fn touch_up(&self, x: u32, y: u32, finger: TouchFinger) -> Result<(), Error>;
}


#[async_trait::async_trait]
impl TouchTrait for ZxTouch {
    async fn show_alert_box(&self, title: &str, content: &str, duration: u32) -> Result<String, Error> {
        if let Some(mut socket) = self.stream.as_ref() {
            let message_type: u8 = MessageType::ShowAlertBox.into();
            let msg = format!("{}{};;{};;{}\r\n", message_type, title, content, duration);
            let mut socket = socket.lock().await;
            match socket
                .write_all(msg.as_bytes()) {
                Ok(_) => {
                    debug!("send message: {}", msg);
                }
                Err(e) => {
                    error!("write error: {}", e);
                }
            }
            let mut buffer = [0u8; 1024];

            socket
                .read(&mut buffer)
                .map(|size| {
                    let msg = String::from_utf8_lossy(&buffer[..size]);
                    debug!("Received message: {}", msg);
                    msg.to_string()
                })
                .map(Ok)
                .map_err(|e| Error::SocketError(e))?
        } else {
            Err(Error::SocketError(std::io::Error::new(std::io::ErrorKind::NotConnected, "not connected")))
        }
    }

    async fn touch_down(&self, x: u32, y: u32, finger: TouchFinger) -> Result<(), Error> {
        self.touch(TouchType::Down, x, y, finger).await
    }

    async fn touch_move(&self, x: u32, y: u32, finger: TouchFinger) -> Result<(), Error> {
        self.touch(TouchType::Move, x, y, finger).await
    }

    async fn touch_up(&self, x: u32, y: u32, finger: TouchFinger) -> Result<(), Error> {
        self.touch(TouchType::Up, x, y, finger).await
    }
}

#[cfg(test)]
mod tests {
    use tracing::level_filters::LevelFilter;
    use crate::zx_touch::{TouchFinger, TouchTrait, ZxTouch};

    fn init_log() {
        let format = tracing_subscriber::fmt::format()
            .with_level(true)
            .with_target(true);

        let sub = tracing_subscriber::fmt()
            .with_max_level(LevelFilter::DEBUG)
            .with_line_number(true)
            .event_format(format);
        sub.init();
    }

    #[tokio::test]
    async fn test_show_alert_box() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        touch.show_alert_box("hello", "hi", 3).await.unwrap();
        touch.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_touch() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        touch.touch_down(200, 200, TouchFinger::Five).await.unwrap();
        touch.close().await.unwrap();
    }
}
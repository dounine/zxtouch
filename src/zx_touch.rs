use crate::error::Error;
use crate::r#type::MessageType;
use futures::lock::Mutex;
use std::fmt::{Debug, Display};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use tracing::{debug, error};

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
    pub(crate) async fn touch(
        &self,
        touch_type: TouchType,
        x: u32,
        y: u32,
        finger: TouchFinger,
    ) -> Result<(), Error> {
        if self.stream.is_none() {
            return Err(Error::SocketError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )));
        }
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::PerformTouch.into();
        let touch_type: u8 = touch_type.into();
        let finger: u8 = finger.into();
        let msg = format!(
            "{};;1{}{}{:05}{:05}\r\n",
            message_type,
            touch_type,
            finger,
            x * 10,
            y * 10
        );
        match socket.write_all(msg.as_bytes()) {
            Ok(_) => {
                debug!("send message: {}", msg);
                Ok(())
            }
            Err(e) => {
                error!("write error: {}", e);
                Err(Error::SocketError(e))
            }
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

#[derive(Debug, Clone, Copy)]
pub(crate) enum TouchType {
    Down,
    Move,
    Up,
}

unsafe impl Send for TouchType {}

unsafe impl Sync for TouchType {}

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
    /// 弹出提示框
    async fn show_alert_box(
        &self,
        title: &str,
        content: &str,
        duration: u32,
    ) -> Result<String, Error>;
    /// 点击屏幕坐标
    async fn touch_down(&self, x: u32, y: u32, finger: TouchFinger) -> Result<(), Error>;
    /// 移动屏幕坐标
    async fn touch_move(&self, x: u32, y: u32, finger: TouchFinger) -> Result<(), Error>;
    /// 抬起屏幕坐标
    async fn touch_up(&self, x: u32, y: u32, finger: TouchFinger) -> Result<(), Error>;
    /// 批量点击
    async fn touch_events(
        &self,
        list: Vec<(TouchType, u32, u32, TouchFinger)>,
    ) -> Result<(), Error>;
    /// 打开app
    async fn switch_to_app(&self, bundle_id: &str) -> Result<String, Error>;
    /// root 方式运行命令
    async fn run_shell_command(&self, command: &str) -> Result<(), Error>;
    /// 图像匹配
    async fn image_match(
        &self,
        image: &str,
        acceptable_value: f32,
        max_try_times: u8,
        scale_ration: f32,
    ) -> Result<(), Error>;
    /// 睡眠
    async fn sleep(&self, microseconds: u32) -> Result<String, Error>;
    /// 显示键盘
    async fn keyboard_show(&self) -> Result<String, Error>;
    /// 隐藏键盘
    async fn keyboard_hide(&self) -> Result<String, Error>;
    /// 输入文本
    async fn text(&self, text: &str) -> Result<String, Error>;
    /// 设置粘贴板内容
    async fn move_cursor(&self, offset: u32) -> Result<String, Error>;
}

#[derive(Debug, Clone)]
enum ParamType {
    String(String),
    I32(i32),
    U32(u32),
    U8(u8),
}

impl Into<String> for ParamType {
    fn into(self) -> String {
        match self {
            ParamType::String(value) => value,
            ParamType::I32(value) => value.to_string(),
            ParamType::U32(value) => value.to_string(),
            ParamType::U8(value) => value.to_string(),
        }
    }
}

impl From<String> for ParamType {
    fn from(value: String) -> Self {
        ParamType::String(value)
    }
}

impl From<i32> for ParamType {
    fn from(value: i32) -> Self {
        ParamType::I32(value)
    }
}

impl From<u32> for ParamType {
    fn from(value: u32) -> Self {
        ParamType::U32(value)
    }
}

impl From<u8> for ParamType {
    fn from(value: u8) -> Self {
        ParamType::U8(value)
    }
}

#[async_trait::async_trait]
impl TouchTrait for ZxTouch {
    async fn show_alert_box(
        &self,
        title: &str,
        content: &str,
        duration: u32,
    ) -> Result<String, Error> {
        if self.stream.is_none() {
            return Err(Error::SocketError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )));
        }
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::ShowAlertBox.into();
        let args: Vec<ParamType> = vec![
            title.to_string().into(),
            content.to_string().into(),
            duration.into(),
        ];
        let args_str = args
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<String>>()
            .join(";;");
        let msg = format!("{}{}\r\n", message_type, args_str);
        match socket.write_all(msg.as_bytes()) {
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

    async fn touch_events(
        &self,
        list: Vec<(TouchType, u32, u32, TouchFinger)>,
    ) -> Result<(), Error> {
        if self.stream.is_none() {
            return Err(Error::SocketError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )));
        }
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::PerformTouch.into();
        let args: Vec<ParamType> = list
            .into_iter()
            .map(|(touch_type, x, y, finger)| {
                let touch_type: u8 = touch_type.into();
                let finger: u8 = finger.into();
                format!("{}{:02}{:05}{:05}", touch_type, finger, x * 10, y * 10).into()
            })
            .collect();
        let args_len = args.len();
        let args_str = args
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<String>>()
            .join("");
        let args_str = format!("{}{}", args_len, args_str);
        let msg = format!("{}{}\r\n", message_type, args_str);
        match socket.write_all(msg.as_bytes()) {
            Ok(_) => {
                debug!("send message: {}", msg);
                Ok(())
            }
            Err(e) => {
                error!("write error: {}", e);
                Err(Error::SocketError(e))
            }
        }
    }

    async fn switch_to_app(&self, bundle_id: &str) -> Result<String, Error> {
        if self.stream.is_none() {
            return Err(Error::SocketError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )));
        }
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::ProcessBringForeground.into();
        let args = format!("{}{}\r\n", message_type, bundle_id);
        match socket.write_all(args.as_bytes()) {
            Ok(_) => {
                debug!("send message: {}", args);
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
    }

    async fn run_shell_command(&self, command: &str) -> Result<(), Error> {
        todo!()
    }

    async fn image_match(
        &self,
        image: &str,
        acceptable_value: f32,
        max_try_times: u8,
        scale_ration: f32,
    ) -> Result<(), Error> {
        todo!()
    }

    async fn sleep(&self, millseconds: u32) -> Result<String, Error> {
        if self.stream.is_none() {
            return Err(Error::SocketError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )));
        }
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::Usleep.into();
        let args = format!("{}{}\r\n", message_type, millseconds * 1000);
        match socket.write_all(args.as_bytes()) {
            Ok(_) => {
                debug!("send message: {}", args);
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
    }

    async fn keyboard_show(&self) -> Result<String, Error> {
        if self.stream.is_none() {
            return Err(Error::SocketError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )));
        }
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::Keyboardimpl.into();
        let args = format!("{}{};;{}\r\n", message_type, 2, 2);
        match socket.write_all(args.as_bytes()) {
            Ok(_) => {
                debug!("send message: {}", args);
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
    }

    async fn keyboard_hide(&self) -> Result<String, Error> {
        if self.stream.is_none() {
            return Err(Error::SocketError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )));
        }
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::Keyboardimpl.into();
        let args = format!("{}{};;{}\r\n", message_type, 2, 1);
        match socket.write_all(args.as_bytes()) {
            Ok(_) => {
                debug!("send message: {}", args);
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
    }

    async fn text(&self, text: &str) -> Result<String, Error> {
        if self.stream.is_none() {
            return Err(Error::SocketError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )));
        }
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::Keyboardimpl.into();
        let args: Vec<ParamType> = vec![1.into(), text.to_string().into()];
        let args_str = args
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<String>>()
            .join(";;");
        let msg = format!("{}{}\r\n", message_type, args_str);
        match socket.write_all(msg.as_bytes()) {
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
    }

    async fn move_cursor(&self, offset: u32) -> Result<String, Error> {
        if self.stream.is_none() {
            return Err(Error::SocketError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )));
        }
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::Keyboardimpl.into();
        let args = format!("{}{};;{}\r\n", message_type, 3, offset);
        match socket.write_all(args.as_bytes()) {
            Ok(_) => {
                debug!("send message: {}", args);
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
    }
}

#[cfg(test)]
mod tests {
    use crate::zx_touch::{TouchFinger, TouchTrait, TouchType, ZxTouch};
    use tracing::level_filters::LevelFilter;

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
        touch.show_alert_box("hello", "hi", 1).await.unwrap();
        touch.sleep(3 * 1000).await.unwrap();
        touch.show_alert_box("hello", "hello", 1).await.unwrap();
        touch.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_touch() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        touch
            .touch_down(329, 2144, TouchFinger::Five)
            .await
            .unwrap();
        touch.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_touch_events() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        touch
            .touch_events(vec![
                (TouchType::Down, 300, 400, TouchFinger::Five),
                (TouchType::Up, 300, 400, TouchFinger::Five),
            ])
            .await
            .unwrap();
        touch.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_switch_to_app() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        touch
            .switch_to_app("com.netskao.dumpdecrypter")
            .await
            .unwrap();
        touch.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_keyboard_show() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        touch.keyboard_show().await.unwrap();
        touch.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_keyboard_hide() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        touch.keyboard_hide().await.unwrap();
        touch.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_sleep() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        touch.keyboard_hide().await.unwrap();
        touch.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_move_cursor() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        touch.move_cursor(3).await.unwrap();
        touch.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_text() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        touch.text("hello").await.unwrap();
        touch.close().await.unwrap();
    }
}

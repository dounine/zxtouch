use crate::error::Error;
use crate::r#type::MessageType;
use futures::lock::Mutex;
use std::fmt::{Debug, Display};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::thread;
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
    pub(crate) async fn basetouch(
        &self,
        touch_type: TouchType,
        x: u32,
        y: u32,
        finger: TouchFinger,
    ) -> Result<(), Error> {
        self.connected_required()?;
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::PerformTouch.into();
        let touch_type: u8 = touch_type.into();
        let finger: u8 = finger.into();
        let msg = format!(
            "{}1{}{:02}{:05}{:05}\r\n",
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

#[derive(Debug, Clone)]
enum ParamType {
    String(String),
    I32(i32),
    U32(u32),
    U8(u8),
}

#[derive(Debug, Clone)]
enum ScreenOrientation {
    Up = 2,    //倒屏
    Down = 1,  //竖屏
    Left = 3,  //左横屏
    Right = 4, //右横屏
}

#[derive(Debug, Clone)]
struct DeviceInfo {
    device_name: String,
    system_name: String,
    system_version: String,
    model: String,
    identifier_for_vendor: String,
}

unsafe impl Send for DeviceInfo {}
unsafe impl Sync for DeviceInfo {}

impl From<i32> for ScreenOrientation {
    fn from(value: i32) -> Self {
        match value {
            0 => ScreenOrientation::Up,
            1 => ScreenOrientation::Down,
            2 => ScreenOrientation::Left,
            3 => ScreenOrientation::Right,
            _ => ScreenOrientation::Down,
        }
    }
}

unsafe impl Send for ScreenOrientation {}
unsafe impl Sync for ScreenOrientation {}

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

impl ZxTouch {
    fn connected_required(&self) -> Result<(), Error> {
        if self.stream.is_none() {
            return Err(Error::SocketError(std::io::Error::new(
                std::io::ErrorKind::NotConnected,
                "not connected",
            )));
        }
        Ok(())
    }

    /// 弹出提示框
    pub async fn show_alert_box(
        &self,
        title: &str,
        content: &str,
        duration: u32,
    ) -> Result<String, Error> {
        self.connected_required()?;
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
    /// 点下屏幕坐标
    pub async fn touch_down(&self, x: u32, y: u32, finger: TouchFinger) -> Result<(), Error> {
        debug!("touch down: {} {} {:?}", x, y, finger);
        self.basetouch(TouchType::Down, x, y, finger).await
    }
    /// 点击屏幕坐标
    pub async fn touch(&self, x: u32, y: u32) -> Result<(), Error> {
        self.touch_events(vec![
            (TouchType::Down, x, y, TouchFinger::Five),
            (TouchType::Up, x, y, TouchFinger::Five),
        ])
        .await
    }
    /// 长按屏幕坐标
    pub async fn touch_long(&self, x: u32, y: u32, duration: u32) -> Result<(), Error> {
        self.touch_down(x, y, TouchFinger::Five).await?;
        self.sleep(duration).await?;
        self.touch_up(x, y, TouchFinger::Five).await
    }

    /// 滑动屏幕坐标
    pub async fn swipe(
        &self,
        x: u32,
        y: u32,
        to_x: u32,
        to_y: u32,
        duration: u32,
    ) -> Result<(), Error> {
        self.touch_down(x, y, TouchFinger::Five).await?;
        if (to_x - x > 50) || (to_y - y > 50) {
            self.sleep(duration / 2).await?;
            self.touch_move(
                (x + (to_x - x) / 2),
                (y + (to_y - y) / 2),
                TouchFinger::Five,
            )
            .await?; //过渡
            self.sleep(duration / 2).await?;
        } else {
            self.sleep(duration).await?;
            self.touch_move(to_x, to_y, TouchFinger::Five).await?;
        }
        self.touch_up(to_x, to_y, TouchFinger::Five).await
    }
    /// 移动屏幕坐标
    pub async fn touch_move(&self, x: u32, y: u32, finger: TouchFinger) -> Result<(), Error> {
        debug!("touch move: {} {} {:?}", x, y, finger);
        self.basetouch(TouchType::Move, x, y, finger).await
    }
    /// 抬起屏幕坐标
    pub async fn touch_up(&self, x: u32, y: u32, finger: TouchFinger) -> Result<(), Error> {
        debug!("touch up: {} {} {:?}", x, y, finger);
        self.basetouch(TouchType::Up, x, y, finger).await
    }
    /// 批量点击
    pub async fn touch_events(
        &self,
        list: Vec<(TouchType, u32, u32, TouchFinger)>,
    ) -> Result<(), Error> {
        self.connected_required()?;
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
    /// 打开app
    pub async fn open_app(&self, bundle_id: &str) -> Result<String, Error> {
        self.connected_required()?;
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
    /// root 方式运行命令
    pub async fn run_shell_command(&self, command: &str) -> Result<String, Error> {
        self.connected_required()?;
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::RunShell.into();
        let args = format!("{}{}\r\n", message_type, command);
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
                debug!("Received message: {} {}", size, msg);
                msg.to_string()
            })
            .map(Ok)
            .map_err(|e| Error::SocketError(e))?
    }
    /// 图像匹配
    pub async fn image_match(
        &self,
        image: &str,
        acceptable_value: f32,
        max_try_times: u8,
        scale_ration: f32,
    ) -> Result<(), Error> {
        todo!()
    }

    /// 睡眠
    pub async fn sleep(&self, millseconds: u32) -> Result<String, Error> {
        debug!("sleep: {}", millseconds);
        self.connected_required()?;
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

    /// 显示键盘
    pub async fn keyboard_show(&self) -> Result<String, Error> {
        self.connected_required()?;
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

    /// 隐藏键盘
    pub async fn keyboard_hide(&self) -> Result<String, Error> {
        self.connected_required()?;
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

    /// 输入文本
    pub async fn text(&self, text: &str) -> Result<String, Error> {
        self.connected_required()?;
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

    /// 设置光标位置
    pub async fn move_cursor(&self, offset: u32) -> Result<String, Error> {
        self.connected_required()?;
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

    ///获取屏幕大小
    pub async fn get_screen_size(&self) -> Result<(i32, i32), Error> {
        self.connected_required()?;
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::GetDeviceInfo.into();
        let args = format!("{}{}\r\n", message_type, 1);
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
                let result = msg.to_string();
                let arr = result.split(";;").collect::<Vec<&str>>();
                (
                    arr[1].split(".").collect::<Vec<_>>()[0].parse().unwrap(),
                    arr[2].split(".").collect::<Vec<_>>()[0].parse().unwrap(),
                )
            })
            .map(Ok)
            .map_err(|e| Error::SocketError(e))?
    }

    /// 获取屏幕方向
    pub async fn get_screen_orientation(&self) -> Result<ScreenOrientation, Error> {
        self.connected_required()?;
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::GetDeviceInfo.into();
        let args = format!("{}{}\r\n", message_type, 2);
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
                msg.split(";;")
                    .map(|x| x.trim())
                    .collect::<Vec<_>>()
                    .get(1)
                    .unwrap()
                    .parse::<i32>()
                    .unwrap()
                    .into()
            })
            .map(Ok)
            .map_err(|e| Error::SocketError(e))?
    }
    /// 获取屏幕缩放比例
    pub async fn get_screen_scale(&self) -> Result<i32, Error> {
        self.connected_required()?;
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::GetDeviceInfo.into();
        let args = format!("{}{}\r\n", message_type, 3);
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
                msg.split(";;")
                    .map(|x| x.trim())
                    .collect::<Vec<_>>()
                    .get(1)
                    .map(|x| x.split(".").collect::<Vec<_>>()[0])
                    .unwrap()
                    .parse::<i32>()
                    .unwrap()
                    .into()
            })
            .map(Ok)
            .map_err(|e| Error::SocketError(e))?
    }

    /// 获取设备信息
    pub async fn get_device_info(&self) -> Result<DeviceInfo, Error> {
        self.connected_required()?;
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::GetDeviceInfo.into();
        let args = format!("{}{}\r\n", message_type, 30);
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
            .map_err(|e| Error::SocketError(e))
            .and_then(|size| {
                let msg = String::from_utf8_lossy(&buffer[..size]);
                debug!("Received message: {}", msg);
                let infos = msg.split(";;").collect::<Vec<&str>>();
                match infos.as_slice() {
                    &[_,device_name, system_name, system_version, model, identifier_for_vendor,..] => {
                        Ok(DeviceInfo {
                            device_name: device_name.to_string(),
                            system_name: system_name.to_string(),
                            system_version: system_version.to_string(),
                            model: model.to_string(),
                            identifier_for_vendor: identifier_for_vendor.trim().to_string(),
                        })
                    }
                    _ => Err(Error::Err("get device info error".to_string())),
                }
            })
    }

    /// 获取支持的ocr语言
    pub async fn get_supported_ocr_languages(
        &self,
        recognition_level: i32,
    ) -> Result<String, Error> {
        self.connected_required()?;
        let mut socket = self.stream.as_ref().unwrap().lock().await;
        let message_type: u8 = MessageType::TextRecognizer.into();
        let args = format!("{}{};;{}\r\n", message_type, 2, recognition_level);
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
    use crate::zx_touch::{TouchFinger, TouchType, ZxTouch};
    use std::thread;
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
    async fn test_touch_down() {
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
    async fn test_open_app() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        touch.open_app("com.netskao.dumpdecrypter").await.unwrap();
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
        touch.show_alert_box("hello", "hi", 1000).await.unwrap();
        touch.sleep(10 * 1000).await.unwrap(); //10秒
        touch.show_alert_box("hello", "hi", 1000).await.unwrap();
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
    async fn test_run_shell_command() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        let result = touch.run_shell_command("pwd").await.unwrap();
        println!("result: {}", result);
        touch.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_screen_size() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        let (width, height) = touch.get_screen_size().await.unwrap();
        println!("iphone width:{width} , height:{height}");
        touch.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_screen_orientation() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        let result = touch.get_screen_orientation().await.unwrap();
        println!("result: {:?}", result);
        touch.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_screen_scale() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        let result = touch.get_screen_scale().await.unwrap();
        println!("result: {}", result);
        touch.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_device_info() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        let result = touch.get_device_info().await.unwrap();
        println!("result: {:?}", result);
        touch.close().await.unwrap();
    }
    #[tokio::test]
    async fn test_get_supported_ocr_languages() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        let result = touch.get_supported_ocr_languages(1).await.unwrap();
        println!("result: {:?}", result);
        touch.close().await.unwrap();
    }
    #[tokio::test]
    async fn test_touch() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        touch.touch(300, 400).await.unwrap();
        touch.close().await.unwrap();
    }
    #[tokio::test]
    async fn test_touch_long() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        touch.touch_long(400, 2247, 500).await.unwrap();
        touch.close().await.unwrap();
    }
    #[tokio::test]
    async fn test_swipe() {
        init_log();
        let mut touch = ZxTouch::new("192.168.3.113", 6000);
        touch.connect().await.unwrap();
        touch.swipe(300, 400, 300, 700, 100).await.unwrap();
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

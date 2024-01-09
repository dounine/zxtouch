//! # zxtouch
//! [![Latest Version](https://img.shields.io/crates/v/zxtouch.svg)](https://crates.io/crates/zxtouch)
//!
//! ios 按键自动化(连点器)、需越狱并已经安装 [zxtouch.deb](https://github.com/dounine/zxtouch/raw/main/deb/com.zjx.ioscontrol_0.0.7-10_iphoneos-arm.deb)
//!
//! ## 功能
//!
//! 1. [显示弹窗](#显示弹窗)
//! 2. [显示toast](#显示toast)
//! 2. [点击屏幕](#点击屏幕)
//! 3. [文本输入](#文本输入)
//! 4. [滑动屏幕](#滑动屏幕)
//! 5. [打开应用](#打开应用)
//! 6. [图像匹配](#图像匹配)
//! 7. [睡眠](#睡眠)
//! 8. [显示键盘](#显示键盘)
//! 9. [隐藏键盘](#隐藏键盘)
//! 10. [设置粘贴板内容](#设置粘贴板内容)
//! 11. [获取粘贴板内容](#获取粘贴板内容)
//! 12. [粘贴](#粘贴)
//! 13. [运行命令](#运行命令)
//!
//! ## 显示弹窗
//!
//! ```rust
//! use zxtouch::zx_touch::{TouchTrait, ZxTouch};
//! let mut touch = ZxTouch::new("192.168.3.113", 6000);//!
//! touch.connect().await.unwrap();
//! touch.show_alert_box("hello", "hi", 3).await.unwrap();
//! ```
//!
//! ## 点击屏幕
//!
//! ```rust
//! use zxtouch::zx_touch::ZxTouch;
//! let mut touch = ZxTouch::new("192.168.3.113", 6000);
//! touch.connect().await.unwrap();
//! touch.touch_down(200, 200, TouchFinger::Five).await.unwrap();
//! touch.close().await.unwrap();
//! ```
//!
//! ## 文本输入
//!
//! ```rust
//! use zxtouch::zx_touch::ZxTouch;
//! let mut touch = ZxTouch::new("192.168.3.113", 6000);
//! touch.connect().await.unwrap();
//! touch.text("hello").await.unwrap();
//! touch.close().await.unwrap();
//! ```


pub mod zx_touch;
pub mod error;
pub mod r#type;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}


#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use tracing::{debug, error};
    use tracing::level_filters::LevelFilter;
    use crate::r#type::MessageType;
    use super::*;

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

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_socket_connect() {
        init_log();
        let host = "192.168.3.113";
        let port = 6000;
        let message_type: u8 = MessageType::ShowAlertBox.into();
        match TcpStream::connect(format!("{}:{}", host, port)) {
            Ok(mut stream) => {
                debug!("Connected to the server!");
                let title = "hello world";
                let content = "hi boy";
                let duration = 3;
                let msg = format!("{}{};;{};;{}\r\n", message_type, title, content, duration);
                stream.write_all(msg.as_bytes()).unwrap();

                //接收消息
                let mut buffer = [0u8; 1024];
                match stream.read(&mut buffer) {
                    Ok(size) => {
                        debug!("Received message: {} : {}",size, String::from_utf8_lossy(&buffer[..size]));
                        //关闭连接
                        //等待3秒
                        std::thread::sleep(std::time::Duration::from_secs(3));
                        stream.write_all(msg.as_bytes()).unwrap();
                    }
                    Err(e) => {
                        error!("Failed to receive data: {}", e)
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect: {}", e)
            }
        }
    }
}

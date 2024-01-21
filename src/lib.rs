/*!
# zxtouch
[![Latest Version](https://img.shields.io/crates/v/zxtouch.svg)](https://crates.io/crates/zxtouch)

ios 按键自动化(连点器)、需越狱并已经安装 [zxtouch.deb](https://github.com/dounine/zxtouch/raw/main/deb/com.zjx.ioscontrol_0.0.7-10_iphoneos-arm.deb)

[![QQ群](https://img.shields.io/badge/QQ%E7%BE%A4-799168925-blue)](http://qm.qq.com/cgi-bin/qm/qr?_wv=1027&k=dLoye8pBcO60zGzqLjGO0l-GgMIaf6wQ&authKey=LfxBdZ5A%2F9eWJbKpzTcuWPjmQu5UdIJ3TVTpqRAQYkCID50WLkYoIXcGxGKzupG3&noverify=0&group_code=799168925)

## 功能

1. [显示弹窗](#显示弹窗)
2. [点击屏幕](#点击屏幕)
3. [长按屏幕](#长按屏幕)
4. [滑动屏幕](#滑动屏幕)
5. [文本输入](#文本输入)
6. [打开应用](#打开应用)
7. [睡眠](#睡眠)
8. [图像匹配](#图像匹配)
9. [显示键盘](#显示键盘)
10. [隐藏键盘](#隐藏键盘)
11. [运行命令](#运行命令)
12. [获取设备信息](#获取设备信息)
13. [设置光标位置](#设置光标位置)
14. [获取屏幕大小](#获取屏幕大小)
15. [获取屏幕方向](#获取屏幕方向)
16. [获取屏幕缩放比例](#获取屏幕缩放比例)

## 显示弹窗

```rust
use zxtouch::zx_touch::{ZxTouch};
let mut touch = ZxTouch::new("192.168.3.113", 6000);//!
touch.connect().await.unwrap();
touch.show_alert_box("hello", "hi", 3).await.unwrap();
touch.close().await.unwrap();
```

## 点击屏幕

```rust
use zxtouch::zx_touch::{TouchFinger, ZxTouch};
let mut touch = ZxTouch::new("192.168.3.113", 6000);
touch.connect().await.unwrap();
touch.touch(200, 200).await.unwrap();
touch.close().await.unwrap();
```

## 长按屏幕

```rust
use zxtouch::zx_touch::{TouchFinger, ZxTouch};
let mut touch = ZxTouch::new("192.168.3.113", 6000);
touch.connect().await.unwrap();
touch.touch_long(200, 200,500).await.unwrap();
touch.close().await.unwrap();
```

## 滑动屏幕

```rust
use zxtouch::zx_touch::{TouchFinger, ZxTouch};
let mut touch = ZxTouch::new("192.168.3.113", 6000);
touch.connect().await.unwrap();
touch.swipe(200, 200, 200, 500, 100).await.unwrap();
touch.close().await.unwrap();
```

## 文本输入

```rust
use zxtouch::zx_touch::ZxTouch;
let mut touch = ZxTouch::new("192.168.3.113", 6000);
touch.connect().await.unwrap();
touch.text("hello").await.unwrap();
touch.close().await.unwrap();
```

## 打开应用

```rust
use zxtouch::zx_touch::ZxTouch;let mut touch = ZxTouch::new("192.168.3.113", 6000);
touch.connect().await.unwrap();
touch
    .open_app("com.netskao.dumpdecrypter")
    .await
    .unwrap();
touch.close().await.unwrap();
```

## 睡眠

```rust
use zxtouch::zx_touch::ZxTouch;let mut touch = ZxTouch::new("192.168.3.113", 6000);
touch.connect().await.unwrap();
touch.show_alert_box("hello", "hi", 1000).await.unwrap();
touch.sleep(10 * 1000).await.unwrap(); //10秒
touch.show_alert_box("hello", "hi", 1000).await.unwrap();
touch.close().await.unwrap();
```

*/

pub mod error;
pub mod r#type;
pub mod zx_touch;
pub mod entity;
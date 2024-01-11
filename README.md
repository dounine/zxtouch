# zxtouch
[![Latest Version](https://img.shields.io/crates/v/zxtouch.svg)](https://crates.io/crates/zxtouch)

ios 按键自动化(连点器)、需越狱并已经安装 [zxtouch.deb](https://github.com/dounine/zxtouch/raw/main/deb/com.zjx.ioscontrol_0.0.7-10_iphoneos-arm.deb)

[![QQ群](https://img.shields.io/badge/QQ%E7%BE%A4-799168925-blue)](http://qm.qq.com/cgi-bin/qm/qr?_wv=1027&k=dLoye8pBcO60zGzqLjGO0l-GgMIaf6wQ&authKey=LfxBdZ5A%2F9eWJbKpzTcuWPjmQu5UdIJ3TVTpqRAQYkCID50WLkYoIXcGxGKzupG3&noverify=0&group_code=799168925)

## 功能

1. [显示弹窗](#显示弹窗)
2. [显示toast](#显示toast)
2. [点击屏幕](#点击屏幕)
3. [文本输入](#文本输入)
4. [滑动屏幕](#滑动屏幕)
5. [打开应用](#打开应用)
6. [图像匹配](#图像匹配)
7. [睡眠](#睡眠)
8. [显示键盘](#显示键盘)
9. [隐藏键盘](#隐藏键盘)
10. [设置粘贴板内容](#设置粘贴板内容)
11. [获取粘贴板内容](#获取粘贴板内容)
12. [粘贴](#粘贴)
13. [运行命令](#运行命令)


## 显示弹窗

```rust
use zxtouch::zx_touch::{TouchTrait, ZxTouch};
let mut touch = ZxTouch::new("192.168.3.113", 6000);//!
touch.connect().await.unwrap();
touch.show_alert_box("hello", "hi", 3).await.unwrap();
```
![screenshot](./image/1.png)
## 点击屏幕

```rust
use zxtouch::zx_touch::ZxTouch;
let mut touch = ZxTouch::new("192.168.3.113", 6000);
touch.connect().await.unwrap();
touch.touch_down(200, 200, TouchFinger::Five).await.unwrap();
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

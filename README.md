# zxtouch
[![Latest Version](https://img.shields.io/crates/v/zxtouch.svg)](https://crates.io/crates/zxtouch)

zxtouch ios 自动化、需要越狱并已经安装zxtouch app

## 功能

1. [控制弹窗](#控制弹窗)


## 控制弹窗

```rust
use zxtouch::zx_touch::{TouchTrait, ZxTouch};
let mut touch = ZxTouch::new("192.168.3.113", 6000);//!
touch.connect().await.unwrap();
touch.show_alert_box("hello", "hi", 3).await.unwrap();
```
![screenshot](./image/1.png)

use zxtouch::zx_touch::ZxTouch;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let mut touch = ZxTouch::new("192.168.3.113", 6000);
    touch.connect().await.unwrap();
    touch.show_alert_box("hello", "hi", 1).await.unwrap();
    touch.sleep(3 * 1000).await.unwrap();
    touch.show_alert_box("hello", "hello", 1).await.unwrap();
    touch.close().await.unwrap();
}

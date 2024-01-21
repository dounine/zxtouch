#[derive(Debug, Clone, Copy)]
pub enum TouchFinger {
    One,
    Two,
    Three,
    Four,
    Five,
}

#[derive(Debug, Clone, Copy)]
pub enum TouchType {
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
pub enum ParamType {
    String(String),
    I32(i32),
    U32(u32),
    U8(u8),
}

#[derive(Debug, Clone)]
pub enum ScreenOrientation {
    Up = 2,    //倒屏
    Down = 1,  //竖屏
    Left = 3,  //左横屏
    Right = 4, //右横屏
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_name: String,
    pub system_name: String,
    pub system_version: String,
    pub model: String,
    pub identifier_for_vendor: String,
}

#[derive(Debug, Clone)]
pub struct FindBuilder {
    pub acceptable: f32,
    pub max_try_times: u8,
    pub scale_ration: f32,
}

unsafe impl Send for FindBuilder {}
unsafe impl Sync for FindBuilder {}

impl FindBuilder {
    pub fn new() -> Self {
        Self {
            acceptable: 0.8,
            max_try_times: 4,
            scale_ration: 0.8,
        }
    }
    pub fn acceptable(&mut self, acceptable: f32) -> &mut Self {
        self.acceptable = acceptable;
        self
    }
    pub fn max_try_times(&mut self, max_try_times: u8) -> &mut Self {
        self.max_try_times = max_try_times;
        self
    }
    pub fn scale_ration(&mut self, scale_ration: f32) -> &mut Self {
        self.scale_ration = scale_ration;
        self
    }

    pub fn build(&self) -> Self {
        Self {
            acceptable: self.acceptable,
            max_try_times: self.max_try_times,
            scale_ration: self.scale_ration,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchInfo {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

unsafe impl Send for MatchInfo {}
unsafe impl Sync for MatchInfo {}

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

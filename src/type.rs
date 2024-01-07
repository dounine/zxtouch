use std::fmt::Display;

pub enum MessageType {
    PerformTouch = 10,
    ProcessBringForeground = 11,
    ShowAlertBox = 12,
    RunShell = 13,
    TouchRecordingStart = 14,
    TouchRecordingStop = 15,
    CrazyTap = 16,
    Depricated = 17,
    Usleep = 18,
    PlayScript = 19,
    PlayScriptForceStop = 20,
    TemplateMatch = 21,
    ShowToast = 22,
    ColorPicker = 23,
    Keyboardimpl = 24,
    GetDeviceInfo = 25,
    TouchIndicator = 26,
    TextRecognizer = 27,
    ColorSearcher = 28,
}

impl From<MessageType> for u8 {
    fn from(value: MessageType) -> Self {
        value as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type() {
        let v: u8 = MessageType::PerformTouch.into();
        assert_eq!(v, 10);
    }
}
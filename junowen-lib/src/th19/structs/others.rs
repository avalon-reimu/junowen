use std::{
    ffi::{CStr, FromBytesUntilNulError},
    fmt,
};

use anyhow::Result;
use derivative::Derivative;
use getset::CopyGetters;

#[derive(Debug)]
#[repr(C)]
pub struct RoundFrame {
    _unknown: [u8; 0x10],
    pub pre_frame: u32,
    pub frame: u32,
}

impl RoundFrame {
    pub fn is_first_frame(&self) -> bool {
        self.pre_frame == 0xffffffff && self.frame == 0
    }
}

#[derive(CopyGetters)]
#[repr(C)]
pub struct VSMode {
    _unknown1: [u8; 0x02E868],
    _unknown2: [u8; 0x08],
    _unknown3: [u8; 0x58],
    player_name: [u8; 0x22],
    room_name: [u8; 0x22],
    _unknown4: [u8; 0x0108],
    /// Readonly
    #[get_copy = "pub"]
    p1_card: u8, // +2ea14h
    /// Readonly
    #[get_copy = "pub"]
    p2_card: u8,
    // unknown remains...
}

impl VSMode {
    pub fn player_name(&self) -> &str {
        CStr::from_bytes_until_nul(&self.player_name)
            .unwrap_or_default()
            .to_str()
            .unwrap()
    }

    pub fn room_name(&self) -> &str {
        CStr::from_bytes_until_nul(&self.room_name)
            .unwrap_or_default()
            .to_str()
            .unwrap()
    }
}

#[derive(CopyGetters)]
pub struct WindowInner {
    #[get_copy = "pub"]
    width: u32,
    #[get_copy = "pub"]
    height: u32,
}

#[derive(Derivative)]
#[derivative(Default)]
#[repr(C)]
pub struct RenderingText {
    #[derivative(Default(value = "[0u8; 256]"))]
    raw_text: [u8; 256],
    x: f32,
    y: f32,
    pub _unknown1: u32,
    /// 0xaarrggbb
    #[derivative(Default(value = "0xffffffff"))]
    pub color: u32,
    #[derivative(Default(value = "1.0"))]
    pub scale_x: f32,
    #[derivative(Default(value = "1.0"))]
    pub scale_y: f32,
    /// radian
    pub rotate: f32,
    pub _unknown2: [u8; 0x08],
    pub font_type: u32,
    pub drop_shadow: bool,
    pub _padding_drop_shadow: [u8; 0x03],
    pub _unknown3: u32,
    pub hide: u32,
    /// 0: center, 1: left, 2: right
    #[derivative(Default(value = "1"))]
    pub horizontal_align: u32,
    /// 0: center, 1: top, 2: bottom
    #[derivative(Default(value = "1"))]
    pub vertical_align: u32,
}

impl fmt::Debug for RenderingText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderingText")
            .field("text", &CStr::from_bytes_until_nul(&self.raw_text))
            .field("x", &self.x)
            .field("y", &self.y)
            .field("_unknown1", &self._unknown1)
            .field("color", &format!("{:x}", self.color))
            .field("scale_x", &self.scale_x)
            .field("scale_y", &self.scale_y)
            .field("rotate", &self.rotate)
            .field("_unknown2", &self._unknown2)
            .field("font_type", &self.font_type)
            .field("drop_shadow", &self.drop_shadow)
            .field("_padding_drop_shadow", &self._padding_drop_shadow)
            .field("_unknown3", &self._unknown3)
            .field("hide", &self.hide)
            .field("horizontal_align", &self.horizontal_align)
            .field("vertical_align", &self.vertical_align)
            .finish()
    }
}

impl RenderingText {
    pub fn text(&self) -> Result<&CStr, FromBytesUntilNulError> {
        CStr::from_bytes_until_nul(&self.raw_text)
    }

    pub fn set_text(&mut self, text: &[u8]) {
        let mut raw_text = [0u8; 256];
        raw_text[0..(text.len())].copy_from_slice(text);
        self.raw_text = raw_text;
    }

    pub fn set_x(&mut self, x: u32, window_inner: &WindowInner) {
        self.x = (x * window_inner.width() / 1280) as f32;
    }

    pub fn set_y(&mut self, y: u32, window_inner: &WindowInner) {
        self.y = (y * window_inner.height() / 960) as f32;
    }

    pub fn sub_y(&mut self, y: u32, window_inner: &WindowInner) {
        self.y -= (y * window_inner.height() / 960) as f32;
    }
}

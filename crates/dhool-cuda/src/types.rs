use dhool_core::FrameTiming;

#[derive(Debug)]
pub struct RenderedFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub timing: FrameTiming,
}

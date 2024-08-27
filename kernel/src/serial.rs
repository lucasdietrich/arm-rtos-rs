pub struct SerialConfig {
    pub baudrate: u32,
}

impl Default for SerialConfig {
    fn default() -> Self {
        SerialConfig { baudrate: 115200 }
    }
}

pub trait SerialTrait {
    fn init(&self, config: &SerialConfig);
    fn write_byte(&self, byte: u8);
    fn read(&self) -> Option<u8>;
}

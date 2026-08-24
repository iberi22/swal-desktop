use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Read;
use std::os::unix::io::AsRawFd;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameCapture {
    pub device_path: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFrame {
    pub data: Vec<u8>,   // Y-channel (luma) de width*height bytes
    pub width: u32,
    pub height: u32,
    pub timestamp_ms: u64,
}

impl FrameCapture {
    pub fn new(device: &str, width: u32, height: u32) -> Self {
        Self {
            device_path: device.to_string(),
            width,
            height,
        }
    }

    /// Captures a real frame from the V4L2 device if available,
    /// or falls back cleanly to synthetic mock data for tests and offline usage.
    pub fn capture_frame(&self) -> RawFrame {
        if let Ok(mut file) = OpenOptions::new().read(true).open(&self.device_path) {
            let buffer_size = (self.width * self.height * 2) as usize; // YUYV 2 bytes per pixel
            let mut raw_buf = vec![0u8; buffer_size];
            if let Ok(n) = file.read(&mut raw_buf) {
                if n >= (self.width * self.height) as usize {
                    // Extract Y (luma) channel: in YUYV, Y is at indices 0, 2, 4...
                    let mut luma = Vec::with_capacity((self.width * self.height) as usize);
                    let mut i = 0;
                    while i < n && luma.len() < (self.width * self.height) as usize {
                        luma.push(raw_buf[i]);
                        i += 2;
                    }
                    while luma.len() < (self.width * self.height) as usize {
                        luma.push(0);
                    }
                    return RawFrame {
                        data: luma,
                        width: self.width,
                        height: self.height,
                        timestamp_ms: current_timestamp_ms(),
                    };
                }
            }
        }

        // Fallback to synthetic frame
        self.capture_mock_frame()
    }

    /// Genera un frame sintético para tests y cuando el dispositivo no está disponible.
    /// Pone valores de piel (140) en la zona central 1/4 del frame.
    pub fn capture_mock_frame(&self) -> RawFrame {
        let size = (self.width * self.height) as usize;
        let mut data = vec![30u8; size]; // fondo oscuro
        // zona central con color de piel (Y=140 = rango de piel)
        let cx = self.width / 2;
        let cy = self.height / 2;
        let radius = self.width.min(self.height) / 6;
        for y in 0..self.height {
            for x in 0..self.width {
                let dx = x as i32 - cx as i32;
                let dy = y as i32 - cy as i32;
                if (dx * dx + dy * dy) < (radius * radius) as i32 {
                    data[(y * self.width + x) as usize] = 140; // skin luma
                }
            }
        }
        RawFrame {
            data,
            width: self.width,
            height: self.height,
            timestamp_ms: current_timestamp_ms(),
        }
    }

    /// Tests if the physical hardware V4L2 device is accessible
    pub fn is_hardware_available(&self) -> bool {
        if let Ok(file) = OpenOptions::new().read(true).open(&self.device_path) {
            let fd = file.as_raw_fd();
            let mut cap = [0u8; 104];
            let res = unsafe { libc::ioctl(fd, 0x80685600, cap.as_mut_ptr()) };
            res == 0
        } else {
            false
        }
    }
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_frame_dimensions() {
        let capture = FrameCapture::new("/dev/video0", 320, 240);
        let frame = capture.capture_mock_frame();
        assert_eq!(frame.width, 320);
        assert_eq!(frame.height, 240);
        assert_eq!(frame.data.len(), 320 * 240);
    }

    #[test]
    fn test_mock_frame_has_skin_pixels() {
        let capture = FrameCapture::new("/dev/video0", 100, 100);
        let frame = capture.capture_mock_frame();
        let skin_count = frame.data.iter().filter(|&&v| v >= 80 && v <= 200).count();
        assert!(skin_count > 0);
    }

    #[test]
    fn test_capture_frame_fallback() {
        let capture = FrameCapture::new("/dev/non_existent_video_device", 64, 64);
        let frame = capture.capture_frame();
        assert_eq!(frame.width, 64);
        assert_eq!(frame.height, 64);
        assert_eq!(frame.data.len(), 64 * 64);
    }
}

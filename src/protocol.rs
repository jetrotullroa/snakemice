//! 90-byte Razer feature reports, as used by the Naga V3 Pro.
//!
//! Verified on the Naga V3 Pro over USB (PIDs 0x00E7 cable / 0x00E8 receiver).

pub const RAZER_VID: u16 = 0x1532;
pub const PID_CABLE: u16 = 0x00E7;
pub const PID_RECEIVER: u16 = 0x00E8;
pub const PID_BLUETOOTH: u16 = 0x00E9;
pub const BLUETOOTH_VID: u16 = 0x068E;

pub const TXN_MOUSE: u8 = 0x1F;
pub const REPORT_LEN: usize = 90;

pub const STATUS_BUSY: u8 = 0x01;
pub const STATUS_OK: u8 = 0x02;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub status: u8,
    pub class: u8,
    pub command: u8,
    pub data_size: u8,
    pub args: [u8; 80],
}

impl Report {
    pub fn command(class: u8, command: u8, data_size: u8, args: &[u8]) -> Self {
        let mut stored = [0u8; 80];
        let n = args.len().min(80);
        stored[..n].copy_from_slice(&args[..n]);
        Self {
            status: 0,
            class,
            command,
            data_size,
            args: stored,
        }
    }

    pub fn to_bytes(&self) -> [u8; REPORT_LEN] {
        let mut raw = [0u8; REPORT_LEN];
        raw[0] = self.status;
        raw[1] = TXN_MOUSE;
        raw[5] = self.data_size;
        raw[6] = self.class;
        raw[7] = self.command;
        raw[8..88].copy_from_slice(&self.args);
        raw[88] = crc(&raw);
        raw
    }

    /// hidapi wants the report id in front of the 90-byte body.
    pub fn to_hid(&self) -> [u8; REPORT_LEN + 1] {
        let mut hid = [0u8; REPORT_LEN + 1];
        hid[1..].copy_from_slice(&self.to_bytes());
        hid
    }
}

pub fn crc(raw: &[u8; REPORT_LEN]) -> u8 {
    raw[2..88].iter().fold(0u8, |acc, byte| acc ^ byte)
}

/// Pick the 90-byte body out of a hidapi feature-report buffer.
/// macOS sometimes keeps the report id in byte 0 and sometimes does not.
pub fn body_from_hid<'a>(buf: &'a [u8], class: u8, command: u8) -> Option<&'a [u8]> {
    let mut found: Option<&[u8]> = None;
    for start in [0usize, 1] {
        if buf.len() < start + REPORT_LEN {
            continue;
        }
        let body = &buf[start..start + REPORT_LEN];
        if body[6] == class && body[7] == command {
            return Some(body);
        }
        if found.is_none() && matches!(body[0], 0x00 | 0x01 | 0x02 | 0x03 | 0x04 | 0x05) {
            found = Some(body);
        }
    }
    found
}

pub fn status_name(status: u8) -> &'static str {
    match status {
        0x00 => "not ready",
        STATUS_BUSY => "busy",
        STATUS_OK => "ok",
        0x03 => "rejected",
        0x04 => "timed out",
        0x05 => "not supported",
        _ => "unknown status",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_covers_bytes_2_through_87() {
        let report = Report::command(0x00, 0x81, 0x02, &[]);
        let raw = report.to_bytes();
        assert_eq!(raw[1], TXN_MOUSE);
        assert_eq!(raw[5], 0x02);
        assert_eq!(raw[6], 0x00);
        assert_eq!(raw[7], 0x81);
        assert_eq!(raw[88], crc(&raw));
        assert_eq!(raw[89], 0);
        let mut expected = 0u8;
        for byte in &raw[2..88] {
            expected ^= byte;
        }
        assert_eq!(raw[88], expected);
    }

    #[test]
    fn hid_buffer_prefixes_report_id_zero() {
        let hid = Report::command(0x02, 0x8C, 0x0A, &[0x01, 0x40, 0x00]).to_hid();
        assert_eq!(hid[0], 0);
        assert_eq!(hid.len(), 91);
        assert_eq!(hid[7], 0x02);
        assert_eq!(hid[8], 0x8C);
        assert_eq!(hid[9], 0x01);
        assert_eq!(hid[10], 0x40);
    }

    #[test]
    fn body_from_hid_accepts_either_alignment() {
        let raw = Report::command(0x07, 0x80, 0x02, &[]).to_bytes();
        let mut prefixed = [0u8; 91];
        prefixed[1..].copy_from_slice(&raw);
        assert_eq!(body_from_hid(&prefixed, 0x07, 0x80).unwrap()[6], 0x07);
        assert_eq!(body_from_hid(&raw, 0x07, 0x80).unwrap()[7], 0x80);
    }
}

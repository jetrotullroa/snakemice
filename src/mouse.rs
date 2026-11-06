//! HID session for the Naga V3 Pro control interface.

use std::ffi::CString;
use std::thread;
use std::time::Duration;

use hidapi::{DeviceInfo, HidApi, HidDevice};

use crate::model::{is_mappable, Binding, BODY, SIX, TWELVE};
use crate::protocol::{
    body_from_hid, status_name, Report, BLUETOOTH_VID, PID_BLUETOOTH, PID_CABLE, PID_RECEIVER,
    RAZER_VID, REPORT_LEN, STATUS_BUSY, STATUS_OK,
};

const PROFILE_STORED: u8 = 0x01;
const PROFILE_LIVE: u8 = 0x00;

pub struct Session {
    api: HidApi,
    mouse: Option<Mouse>,
    keyboards: Vec<HidDevice>,
    pub transport: String,
    pub firmware: String,
    pub detail: String,
}

struct Mouse {
    device: HidDevice,
    dpi_storage: u8,
}

#[derive(Clone, Debug)]
pub struct Reading {
    pub transport: String,
    pub firmware: String,
    pub battery: Option<u8>,
    pub charging: Option<bool>,
    pub dpi_x: Option<u16>,
    pub dpi_y: Option<u16>,
    pub poll_hz: Option<u16>,
    pub buttons: Vec<(u8, Binding)>,
    pub warning: String,
}

impl Session {
    pub fn new() -> Result<Self, String> {
        let api = HidApi::new().map_err(|err| err.to_string())?;
        Ok(Self {
            api,
            mouse: None,
            keyboards: Vec::new(),
            transport: String::new(),
            firmware: String::new(),
            detail: String::new(),
        })
    }

    pub fn is_open(&self) -> bool {
        self.mouse.is_some()
    }

    pub fn close(&mut self) {
        self.mouse = None;
        self.keyboards.clear();
    }

    pub fn scan(&mut self) -> Result<Option<Reading>, String> {
        self.close();
        self.api.refresh_devices().map_err(|err| err.to_string())?;
        let seen = describe_razer(&self.api);
        let cable = candidates(&self.api, PID_CABLE);
        let wireless = candidates(&self.api, PID_RECEIVER);
        if cable.is_empty() && wireless.is_empty() {
            self.detail = if seen.is_empty() {
                "Nothing from Razer is on USB. Plug in the cable or the wireless receiver, and quit Synapse if it is open.".into()
            } else if seen.contains("Bluetooth") {
                format!("Bluetooth is connected, and this app can't configure the mouse that way.\nPlug in the cable or the USB receiver.\n\n{seen}")
            } else {
                format!("No Naga V3 Pro control interface yet.\n\n{seen}")
            };
            return Ok(None);
        }

        let mut opened = try_open(&self.api, &cable);
        let mut transport = "Cable";
        if opened.is_none() {
            opened = try_open(&self.api, &wireless);
            transport = "Wireless receiver";
        }
        let Some((device, pid)) = opened else {
            self.detail = format!("The Naga V3 Pro is plugged in, but it didn't answer.\nQuit Razer Synapse and try the cable.\n\n{seen}");
            return Ok(None);
        };

        self.transport = transport.to_string();
        self.firmware = read_firmware(&device).unwrap_or_else(|_| "unknown".into());
        self.keyboards = open_keyboards(&self.api, pid);
        self.mouse = Some(Mouse {
            device,
            dpi_storage: PROFILE_STORED,
        });
        self.detail.clear();
        self.read_all()
    }

    pub fn read_all(&mut self) -> Result<Option<Reading>, String> {
        let Some(mouse) = self.mouse.as_mut() else {
            return Ok(None);
        };
        let firmware = read_firmware(&mouse.device)?;
        let battery = read_battery(&mouse.device).ok();
        let charging = read_charging(&mouse.device).ok();
        let (storage, dpi_x, dpi_y) =
            read_dpi(&mouse.device).unwrap_or((PROFILE_STORED, None, None));
        mouse.dpi_storage = storage;
        let poll_hz = read_poll(&mouse.device).ok().flatten();
        let mut buttons = Vec::with_capacity(23);
        let mut warning = String::new();
        for spec in TWELVE
            .iter()
            .chain(SIX.iter())
            .map(|button| button.slot)
            .chain(BODY.iter().map(|button| button.slot))
        {
            match read_button(&mouse.device, spec) {
                Ok(binding) => buttons.push((spec, binding)),
                Err(err) if warning.is_empty() => {
                    warning = format!("Slot {spec} didn't answer: {err}");
                }
                Err(_) => {}
            }
        }
        self.firmware = firmware.clone();
        Ok(Some(Reading {
            transport: self.transport.clone(),
            firmware,
            battery,
            charging,
            dpi_x,
            dpi_y,
            poll_hz,
            buttons,
            warning,
        }))
    }

    pub fn poll_power(&mut self) -> Result<(Option<u8>, Option<bool>), String> {
        let mouse = self.mouse.as_ref().ok_or("Mouse is not connected.")?;
        let battery = read_battery(&mouse.device)?;
        let charging = read_charging(&mouse.device).ok();
        Ok((Some(battery), charging))
    }

    pub fn write_button(&mut self, slot: u8, binding: &Binding) -> Result<Binding, String> {
        let mouse = self.mouse.as_ref().ok_or("Mouse is not connected.")?;
        let stored = binding.encode(slot, PROFILE_STORED);
        write_button(&mouse.device, &stored)?;
        let live = binding.encode(slot, PROFILE_LIVE);
        if write_button(&mouse.device, &live).is_err() {
            // The stored profile is what the plate keeps using with the app closed.
        }
        read_button(&mouse.device, slot)
    }

    pub fn set_dpi(&mut self, x: u16, y: u16) -> Result<(u16, u16), String> {
        let mouse = self.mouse.as_mut().ok_or("Mouse is not connected.")?;
        let x = x.clamp(100, 50_000);
        let y = y.clamp(100, 50_000);
        let args = [
            mouse.dpi_storage,
            (x >> 8) as u8,
            (x & 0xFF) as u8,
            (y >> 8) as u8,
            (y & 0xFF) as u8,
            0,
            0,
        ];
        query(&mouse.device, 0x04, 0x05, 0x07, &args)?;
        let (_, rx, ry) = read_dpi(&mouse.device)?;
        Ok((rx.unwrap_or(x), ry.unwrap_or(y)))
    }

    pub fn set_poll(&mut self, hz: u16) -> Result<u16, String> {
        let mouse = self.mouse.as_ref().ok_or("Mouse is not connected.")?;
        let code = match hz {
            1000 => 0x01u8,
            500 => 0x02,
            125 => 0x08,
            _ => return Err("Polling rate must be 125, 500, or 1000 Hz.".into()),
        };
        query(&mouse.device, 0x00, 0x05, 0x01, &[code])?;
        read_poll(&mouse.device)?.ok_or_else(|| "The mouse didn't report a polling rate.".into())
    }

    pub fn poll_keys(&mut self) -> Vec<u8> {
        let mut pressed = Vec::new();
        for device in &self.keyboards {
            let mut buf = [0u8; 32];
            match device.read_timeout(&mut buf, 5) {
                Ok(0) | Err(_) => continue,
                Ok(n) => pressed.extend(keyboard_keys(&buf[..n])),
            }
        }
        pressed
    }
}

fn candidates(api: &HidApi, pid: u16) -> Vec<CString> {
    let mut rows: Vec<(u8, CString)> = api
        .device_list()
        .filter(|device| device.vendor_id() == RAZER_VID && device.product_id() == pid)
        .map(|device| {
            let rank = if device.usage_page() == 0x01 && device.usage() == 0x03 {
                0
            } else if device.usage_page() == 0x01 && device.usage() == 0x02 {
                2
            } else if device.usage_page() == 0x01 && device.usage() == 0x06 {
                3
            } else {
                1
            };
            (rank, device.path().to_owned())
        })
        .collect();
    rows.sort_by_key(|(rank, _)| *rank);
    rows.into_iter().map(|(_, path)| path).collect()
}

fn try_open(api: &HidApi, paths: &[CString]) -> Option<(HidDevice, u16)> {
    for path in paths {
        let Ok(device) = api.open_path(path) else {
            continue;
        };
        let _ = device.set_blocking_mode(true);
        if read_firmware(&device).is_ok() {
            let pid = api
                .device_list()
                .find(|info| info.path() == path.as_c_str())
                .map(DeviceInfo::product_id)
                .unwrap_or(PID_CABLE);
            return Some((device, pid));
        }
    }
    None
}

fn open_keyboards(api: &HidApi, pid: u16) -> Vec<HidDevice> {
    api.device_list()
        .filter(|device| {
            device.vendor_id() == RAZER_VID
                && device.product_id() == pid
                && device.usage_page() == 0x01
                && device.usage() == 0x06
        })
        .filter_map(|device| {
            let opened = api.open_path(device.path()).ok()?;
            opened.set_blocking_mode(false).ok()?;
            Some(opened)
        })
        .collect()
}

fn describe_razer(api: &HidApi) -> String {
    let mut lines = Vec::new();
    for device in api.device_list() {
        let vid = device.vendor_id();
        if vid != RAZER_VID && vid != BLUETOOTH_VID {
            continue;
        }
        let name = device.product_string().unwrap_or("Razer device");
        let bluetooth = vid == BLUETOOTH_VID || device.product_id() == PID_BLUETOOTH;
        let kind = if bluetooth { "Bluetooth" } else { "USB" };
        let line = format!(
            "{kind}  {vid:04X}:{:04X}  {name}  usage {:04X}/{:04X}",
            device.product_id(),
            device.usage_page(),
            device.usage()
        );
        if !lines.contains(&line) {
            lines.push(line);
        }
    }
    lines.join("\n")
}

fn query(
    device: &HidDevice,
    class: u8,
    command: u8,
    size: u8,
    args: &[u8],
) -> Result<Vec<u8>, String> {
    let hid = Report::command(class, command, size, args).to_hid();
    device
        .send_feature_report(&hid)
        .map_err(|err| format!("Couldn't send to the mouse: {err}"))?;
    thread::sleep(Duration::from_millis(45));
    for _ in 0..16 {
        let mut buf = [0u8; REPORT_LEN + 1];
        match device.get_feature_report(&mut buf) {
            Ok(n) => {
                if let Some(body) = body_from_hid(&buf[..n.max(1).min(buf.len())], class, command) {
                    if body[0] == STATUS_BUSY {
                        thread::sleep(Duration::from_millis(25));
                        continue;
                    }
                    if body[0] == STATUS_OK {
                        return Ok(body.to_vec());
                    }
                    return Err(format!(
                        "The mouse answered {status} for {class:02X}:{command:02X}.",
                        status = status_name(body[0])
                    ));
                }
            }
            Err(err) => return Err(format!("Couldn't read the mouse: {err}")),
        }
        thread::sleep(Duration::from_millis(25));
    }
    Err("The mouse didn't answer in time. On wireless, plug in the cable.".into())
}

fn read_firmware(device: &HidDevice) -> Result<String, String> {
    let body = query(device, 0x00, 0x81, 0x02, &[])?;
    let major = body[8];
    let minor = body[9];
    if body[10] == 0 && body[11] == 0 {
        Ok(format!("{major}.{minor:02}"))
    } else {
        Ok(format!("{major}.{minor:02}.{}.{:02}", body[10], body[11]))
    }
}

fn read_battery(device: &HidDevice) -> Result<u8, String> {
    let body = query(device, 0x07, 0x80, 0x02, &[])?;
    Ok(((u16::from(body[9]) * 100 + 127) / 255) as u8)
}

fn read_charging(device: &HidDevice) -> Result<bool, String> {
    let body = query(device, 0x07, 0x84, 0x02, &[])?;
    Ok(body[9] == 1)
}

fn read_dpi(device: &HidDevice) -> Result<(u8, Option<u16>, Option<u16>), String> {
    for storage in [PROFILE_STORED, PROFILE_LIVE] {
        if let Ok(body) = query(device, 0x04, 0x85, 0x07, &[storage]) {
            let x = u16::from_be_bytes([body[9], body[10]]);
            let y = u16::from_be_bytes([body[11], body[12]]);
            if (100..=50_000).contains(&x) && (100..=50_000).contains(&y) {
                return Ok((storage, Some(x), Some(y)));
            }
        }
    }
    Ok((PROFILE_STORED, None, None))
}

fn read_poll(device: &HidDevice) -> Result<Option<u16>, String> {
    let body = query(device, 0x00, 0x85, 0x01, &[])?;
    Ok(match body[8] {
        0x01 => Some(1000),
        0x02 => Some(500),
        0x08 => Some(125),
        _ => None,
    })
}

fn read_button(device: &HidDevice, slot: u8) -> Result<Binding, String> {
    let body = query(device, 0x02, 0x8C, 0x0A, &[PROFILE_STORED, slot, 0x00])?;
    let args = body.get(8..18).ok_or("Short button reply")?;
    let binding = Binding::decode(args);
    if !is_mappable(slot) {
        return Err(format!("Slot {slot} is not mapped in SnakeMice."));
    }
    Ok(binding)
}

fn write_button(device: &HidDevice, args: &[u8; 10]) -> Result<(), String> {
    let body = query(device, 0x02, 0x0C, 0x0A, args)?;
    let echo = body.get(8..18).unwrap_or(&[]);
    if echo != args {
        return Err("The mouse didn't confirm that button.".into());
    }
    Ok(())
}

fn keyboard_keys(buf: &[u8]) -> Vec<u8> {
    let data = if buf.len() >= 9 && buf[0] <= 0x04 {
        &buf[1..]
    } else {
        buf
    };
    if data.len() < 8 || data[1] != 0 {
        return Vec::new();
    }
    data[2..8]
        .iter()
        .copied()
        .filter(|key| (0x04..0xE8).contains(key))
        .collect()
}

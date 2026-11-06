//! Side-button slots and the bindings stored in the mouse.

use std::collections::HashMap;

/// Which magnetic plate a slot belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plate {
    Twelve,
    Six,
}

impl Plate {
    pub fn label(self) -> &'static str {
        match self {
            Plate::Twelve => "12-button plate",
            Plate::Six => "6-button plate",
        }
    }

    pub fn columns(self) -> u8 {
        match self {
            Plate::Twelve => 3,
            Plate::Six => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SideButton {
    pub slot: u8,
    pub number: u8,
    pub factory_key: u8,
    pub plate: Plate,
}

/// 12-button plate, printed labels 1–12. Slots 0x40–0x4B.
/// Factory keys are 1–9, 0, -, =.
pub const TWELVE: [SideButton; 12] = buttons(Plate::Twelve, 0x40, TWELVE_KEYS);

/// 6-button plate, printed labels 1–6. Slots 0x50–0x55.
/// Factory keys are 1–6. The mouse uses these slots on its own when that plate is snapped on.
pub const SIX: [SideButton; 6] = buttons(Plate::Six, 0x50, SIX_KEYS);

const TWELVE_KEYS: [u8; 12] = [
    0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x2D, 0x2E,
];
const SIX_KEYS: [u8; 6] = [0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23];

const fn buttons<const N: usize>(plate: Plate, first_slot: u8, keys: [u8; N]) -> [SideButton; N] {
    let mut out = [SideButton {
        slot: 0,
        number: 0,
        factory_key: 0,
        plate,
    }; N];
    let mut i = 0;
    while i < N {
        out[i] = SideButton {
            slot: first_slot + i as u8,
            number: i as u8 + 1,
            factory_key: keys[i],
            plate,
        };
        i += 1;
    }
    out
}

pub fn button_for(slot: u8) -> Option<&'static SideButton> {
    TWELVE
        .iter()
        .chain(SIX.iter())
        .find(|button| button.slot == slot)
}

/// Top-shell and grip buttons (never on the magnetic side plate). Slot IDs for the Naga V3 Pro.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyButton {
    pub slot: u8,
    pub name: &'static str,
    pub detail: &'static str,
    pub factory: Binding,
}

pub const BODY: [BodyButton; 5] = [
    BodyButton {
        slot: 11,
        name: "Top edge · front",
        detail: "On the top shell, left of left-click (quick-access edge) — factory DPI stage up",
        factory: Binding::DpiStageUp,
    },
    BodyButton {
        slot: 12,
        name: "Top edge · rear",
        detail: "On the top shell, left of left-click (quick-access edge) — factory DPI stage down",
        factory: Binding::DpiStageDown,
    },
    BodyButton {
        slot: 106,
        name: "Top · wheel front",
        detail: "On the top shell, in front of the scroll wheel — factory scroll mode",
        factory: Binding::ScrollModeCycle,
    },
    BodyButton {
        slot: 128,
        name: "Top · wheel rear",
        detail: "On the top shell, behind the scroll wheel — factory: no action",
        factory: Binding::Disabled,
    },
    BodyButton {
        slot: 57,
        name: "Ring finger strip",
        detail: "Right grip (lockable ring-finger button), not the side plate — factory HyperShift",
        factory: Binding::HyperShift,
    },
];

pub fn body_for(slot: u8) -> Option<&'static BodyButton> {
    BODY.iter().find(|button| button.slot == slot)
}

pub fn is_mappable(slot: u8) -> bool {
    button_for(slot).is_some() || body_for(slot).is_some()
}

/// Ctrl on the left edge, Alt on the rear edge, DPI under the wheel, Shift on the ring finger.
pub fn modifier_edge_layout() -> Vec<(u8, Binding)> {
    vec![
        (11, Binding::left_control()),
        (12, Binding::left_alt()),
        (106, Binding::DpiStageUp),
        (128, Binding::DpiStageDown),
        (57, Binding::left_shift()),
    ]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Binding {
    Keyboard {
        modifiers: u8,
        key: u8,
    },
    Mouse(u8),
    Disabled,
    DpiStageUp,
    DpiStageDown,
    ScrollModeCycle,
    HyperShift,
    /// A vendor action we don't rewrite unless the user picks something else.
    Special(Vec<u8>),
}

impl Binding {
    pub const HID_LEFT_CTRL: u8 = 0xE0;
    pub const HID_LEFT_SHIFT: u8 = 0xE1;
    pub const HID_LEFT_ALT: u8 = 0xE2;
    pub const HID_LEFT_GUI: u8 = 0xE3;

    pub fn factory(button: &SideButton) -> Self {
        Self::Keyboard {
            modifiers: 0,
            key: button.factory_key,
        }
    }

    pub fn left_control() -> Self {
        Self::Keyboard {
            modifiers: 0,
            key: Self::HID_LEFT_CTRL,
        }
    }

    pub fn left_shift() -> Self {
        Self::Keyboard {
            modifiers: 0,
            key: Self::HID_LEFT_SHIFT,
        }
    }

    pub fn left_alt() -> Self {
        Self::Keyboard {
            modifiers: 0,
            key: Self::HID_LEFT_ALT,
        }
    }

    pub fn is_factory(&self, button: &SideButton) -> bool {
        matches!(
            self,
            Self::Keyboard { modifiers: 0, key } if *key == button.factory_key
        )
    }

    pub fn is_body_factory(&self, button: &BodyButton) -> bool {
        self == &button.factory
    }

    /// 10 argument bytes: profile, slot, layer, type, length, data…
    pub fn encode(&self, slot: u8, profile: u8) -> [u8; 10] {
        let mut args = [0u8; 10];
        args[0] = profile;
        args[1] = slot;
        args[2] = 0x00;
        match self {
            Self::Keyboard { modifiers, key } => {
                args[3] = 0x02;
                args[4] = 0x02;
                args[5] = *modifiers & 0x0F;
                args[6] = *key;
            }
            Self::Mouse(button) => {
                args[3] = 0x01;
                args[4] = 0x01;
                args[5] = *button;
            }
            Self::Disabled => {
                args[3] = 0x00;
                args[4] = 0x01;
                args[5] = 0x00;
            }
            Self::DpiStageUp => {
                args[3] = 0x06;
                args[4] = 0x01;
                args[5] = 0x01;
            }
            Self::DpiStageDown => {
                args[3] = 0x06;
                args[4] = 0x01;
                args[5] = 0x02;
            }
            Self::ScrollModeCycle => {
                args[3] = 0x12;
                args[4] = 0x01;
                args[5] = 0x01;
            }
            Self::HyperShift => {
                args[3] = 0x0C;
                args[4] = 0x01;
                args[5] = 0x39;
            }
            Self::Special(raw) => {
                if raw.len() >= 3 && raw[0] == profile && raw[1] == slot {
                    let n = raw.len().min(10);
                    args[..n].copy_from_slice(&raw[..n]);
                } else {
                    Self::write_function_block(&mut args, raw);
                }
            }
        }
        args
    }

    fn write_function_block(args: &mut [u8; 10], raw: &[u8]) {
        if raw.len() >= 10 {
            args.copy_from_slice(&raw[..10]);
            return;
        }
        if raw.len() >= 2 {
            args[3] = raw[0];
            args[4] = raw[1];
            let len = usize::min(raw[1] as usize, 5);
            if raw.len() >= 2 + len {
                args[5..5 + len].copy_from_slice(&raw[2..2 + len]);
            }
        }
    }

    pub fn decode(args: &[u8]) -> Self {
        if args.len() < 7 {
            return Self::Special(args.to_vec());
        }
        match args[3] {
            0x02 => Self::Keyboard {
                modifiers: args[5],
                key: args[6],
            },
            0x01 => Self::Mouse(args[5]),
            0x00 if args[4] == 0x01 && args[5] == 0x00 => Self::Disabled,
            0x06 if args[4] == 0x01 && args[5] == 0x01 => Self::DpiStageUp,
            0x06 if args[4] == 0x01 && args[5] == 0x02 => Self::DpiStageDown,
            0x12 if args[4] == 0x01 && args[5] == 0x01 => Self::ScrollModeCycle,
            0x0C if args[4] == 0x01 && args[5] == 0x39 => Self::HyperShift,
            _ => Self::Special(args[..10.min(args.len())].to_vec()),
        }
    }

    pub fn summary(&self) -> String {
        match self {
            Self::Keyboard { modifiers, key } => {
                format!("{}{}", modifier_label(*modifiers), key_name(*key))
            }
            Self::Mouse(button) => mouse_name(*button).to_string(),
            Self::Disabled => "Off".to_string(),
            Self::DpiStageUp => "DPI stage up".to_string(),
            Self::DpiStageDown => "DPI stage down".to_string(),
            Self::ScrollModeCycle => "Scroll mode".to_string(),
            Self::HyperShift => "HyperShift".to_string(),
            Self::Special(_) => "Factory function".to_string(),
        }
    }
}

pub fn modifier_label(modifiers: u8) -> String {
    let mut label = String::new();
    if modifiers & 0x11 != 0 {
        label.push('⌃');
    }
    if modifiers & 0x44 != 0 {
        label.push('⌥');
    }
    if modifiers & 0x22 != 0 {
        label.push('⇧');
    }
    if modifiers & 0x88 != 0 {
        label.push('⌘');
    }
    label
}

pub fn key_name(key: u8) -> String {
    match key {
        0xE0 => "Left Control".to_string(),
        0xE1 => "Left Shift".to_string(),
        0xE2 => "Left Alt (Option)".to_string(),
        0xE3 => "Left Command".to_string(),
        0xE4 => "Right Control".to_string(),
        0xE5 => "Right Shift".to_string(),
        0xE6 => "Right Alt".to_string(),
        0xE7 => "Right Command".to_string(),
        0x04..=0x1D => ((b'A' + (key - 0x04)) as char).to_string(),
        0x1E..=0x26 => (key - 0x1D).to_string(),
        0x27 => "0".to_string(),
        0x28 => "Return".to_string(),
        0x29 => "Esc".to_string(),
        0x2A => "Delete".to_string(),
        0x2B => "Tab".to_string(),
        0x2C => "Space".to_string(),
        0x2D => "-".to_string(),
        0x2E => "=".to_string(),
        0x2F => "[".to_string(),
        0x30 => "]".to_string(),
        0x31 => "\\".to_string(),
        0x33 => ";".to_string(),
        0x34 => "'".to_string(),
        0x35 => "`".to_string(),
        0x36 => ",".to_string(),
        0x37 => ".".to_string(),
        0x38 => "/".to_string(),
        0x3A..=0x45 => format!("F{}", key - 0x39),
        0x4A => "Home".to_string(),
        0x4B => "Page Up".to_string(),
        0x4C => "Fwd Delete".to_string(),
        0x4D => "End".to_string(),
        0x4E => "Page Down".to_string(),
        0x4F => "Right".to_string(),
        0x50 => "Left".to_string(),
        0x51 => "Down".to_string(),
        0x52 => "Up".to_string(),
        0x54 => "Num /".to_string(),
        0x55 => "Num *".to_string(),
        0x56 => "Num -".to_string(),
        0x57 => "Num +".to_string(),
        0x58 => "Num Enter".to_string(),
        0x59..=0x61 => format!("Num {}", key - 0x58),
        0x62 => "Num 0".to_string(),
        0x63 => "Num .".to_string(),
        0x68..=0x73 => format!("F{}", key - 0x68 + 13),
        _ => format!("Key {key:02X}"),
    }
}

pub fn mouse_name(button: u8) -> &'static str {
    match button {
        1 => "Left click",
        2 => "Right click",
        3 => "Middle click",
        4 => "Back",
        5 => "Forward",
        9 => "Wheel up",
        10 => "Wheel down",
        _ => "Mouse button",
    }
}

pub const MODIFIER_KEYS: [(&str, u8); 4] = [
    ("Left Control", Binding::HID_LEFT_CTRL),
    ("Left Shift", Binding::HID_LEFT_SHIFT),
    ("Left Alt (Option)", Binding::HID_LEFT_ALT),
    ("Left Command", Binding::HID_LEFT_GUI),
];

pub const MOUSE_CHOICES: [(u8, &str); 7] = [
    (1, "Left click"),
    (2, "Right click"),
    (3, "Middle click"),
    (4, "Back"),
    (5, "Forward"),
    (9, "Wheel up"),
    (10, "Wheel down"),
];

/// A side-button key that belongs to only one plate tells us which plate is attached.
pub fn plate_from_key(key: u8, bindings: &HashMap<u8, Binding>) -> Option<Plate> {
    let mut found: Option<Plate> = None;
    for button in TWELVE.iter().chain(SIX.iter()) {
        let Some(Binding::Keyboard {
            modifiers,
            key: bound,
        }) = bindings.get(&button.slot)
        else {
            continue;
        };
        if *modifiers & 0x0F != 0 || *bound != key {
            continue;
        }
        match found {
            None => found = Some(button.plate),
            Some(plate) if plate != button.plate => return None,
            Some(_) => {}
        }
    }
    found
}

/// Map a DOM `KeyboardEvent.code` value to a USB HID keyboard usage.
pub fn hid_from_code(code: &str) -> Option<u8> {
    if let Some(rest) = code.strip_prefix("Key") {
        let ch = rest.chars().next()?;
        if rest.len() == 1 && ch.is_ascii_uppercase() {
            return Some(0x04 + (ch as u8 - b'A'));
        }
    }
    if let Some(rest) = code.strip_prefix("Digit") {
        let ch = rest.chars().next()?;
        if rest.len() == 1 && ch.is_ascii_digit() {
            return Some(if ch == '0' {
                0x27
            } else {
                0x1E + (ch as u8 - b'1')
            });
        }
    }
    if let Some(rest) = code.strip_prefix('F') {
        if let Ok(n) = rest.parse::<u8>() {
            return match n {
                1..=12 => Some(0x3A + n - 1),
                13..=24 => Some(0x68 + n - 13),
                _ => None,
            };
        }
    }
    Some(match code {
        "Minus" => 0x2D,
        "Equal" => 0x2E,
        "BracketLeft" => 0x2F,
        "BracketRight" => 0x30,
        "Backslash" => 0x31,
        "Semicolon" => 0x33,
        "Quote" => 0x34,
        "Backquote" => 0x35,
        "Comma" => 0x36,
        "Period" => 0x37,
        "Slash" => 0x38,
        "Enter" | "NumpadEnter" => 0x28,
        "Escape" => 0x29,
        "Backspace" => 0x2A,
        "Tab" => 0x2B,
        "Space" => 0x2C,
        "Home" => 0x4A,
        "PageUp" => 0x4B,
        "Delete" => 0x4C,
        "End" => 0x4D,
        "PageDown" => 0x4E,
        "ArrowRight" => 0x4F,
        "ArrowLeft" => 0x50,
        "ArrowDown" => 0x51,
        "ArrowUp" => 0x52,
        "NumpadDivide" => 0x54,
        "NumpadMultiply" => 0x55,
        "NumpadSubtract" => 0x56,
        "NumpadAdd" => 0x57,
        "Numpad1" => 0x59,
        "Numpad2" => 0x5A,
        "Numpad3" => 0x5B,
        "Numpad4" => 0x5C,
        "Numpad5" => 0x5D,
        "Numpad6" => 0x5E,
        "Numpad7" => 0x5F,
        "Numpad8" => 0x60,
        "Numpad9" => 0x61,
        "Numpad0" => 0x62,
        "NumpadDecimal" => 0x63,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plates_use_distinct_slots() {
        assert_eq!(TWELVE[0].slot, 0x40);
        assert_eq!(TWELVE[11].slot, 0x4B);
        assert_eq!(TWELVE[11].factory_key, 0x2E);
        assert_eq!(SIX[0].slot, 0x50);
        assert_eq!(SIX[5].slot, 0x55);
        assert_eq!(SIX[5].factory_key, 0x23);
        assert!(TWELVE.iter().all(|b| b.plate == Plate::Twelve));
        assert!(SIX.iter().all(|b| b.plate == Plate::Six));
    }

    #[test]
    fn body_slots_and_modifier_layout() {
        assert_eq!(BODY[0].slot, 11);
        assert_eq!(BODY[4].slot, 57);
        let layout = modifier_edge_layout();
        assert_eq!(layout.len(), 5);
        assert_eq!(layout[0].1, Binding::left_control());
        assert_eq!(layout[4].1, Binding::left_shift());
    }

    #[test]
    fn dpi_and_modifiers_encode() {
        let up = Binding::DpiStageUp.encode(11, 0x01);
        assert_eq!(up[1], 11);
        assert_eq!(up[3], 0x06);
        assert_eq!(up[5], 0x01);
        assert_eq!(Binding::decode(&up), Binding::DpiStageUp);
        let ctrl = Binding::left_control().encode(11, 0x01);
        assert_eq!(ctrl[6], 0xE0);
        assert_eq!(
            Binding::decode(&ctrl).summary(),
            "Left Control"
        );
    }

    #[test]
    fn factory_roundtrip_and_synapse_length() {
        let args = Binding::factory(&TWELVE[0]).encode(0x40, 0x01);
        assert_eq!(args, [0x01, 0x40, 0x00, 0x02, 0x02, 0x00, 0x1E, 0, 0, 0]);
        assert_eq!(
            Binding::decode(&args),
            Binding::Keyboard {
                modifiers: 0,
                key: 0x1E
            }
        );
        let stored = [0x01, 0x40, 0x00, 0x02, 0x01, 0x00, 0x1E, 0, 0, 0];
        assert!(Binding::decode(&stored).is_factory(&TWELVE[0]));
    }

    #[test]
    fn modifiers_and_mouse_buttons() {
        let binding = Binding::Keyboard {
            modifiers: 0x01 | 0x08,
            key: 0x3A,
        };
        assert_eq!(binding.summary(), "⌃⌘F1");
        assert_eq!(binding.encode(0x41, 0x01)[3], 0x02);
        assert_eq!(Binding::Mouse(5).summary(), "Forward");
        assert_eq!(
            Binding::decode(&[0x01, 0x50, 0, 0x01, 0x01, 0x05, 0, 0, 0, 0]),
            Binding::Mouse(5)
        );
    }

    #[test]
    fn unique_key_selects_the_plate() {
        let mut map = HashMap::new();
        for button in TWELVE {
            map.insert(button.slot, Binding::factory(&button));
        }
        for button in SIX {
            map.insert(button.slot, Binding::factory(&button));
        }
        assert_eq!(plate_from_key(0x24, &map), Some(Plate::Twelve));
        assert_eq!(plate_from_key(0x2E, &map), Some(Plate::Twelve));
        assert_eq!(plate_from_key(0x1E, &map), None);

        map.insert(
            SIX[0].slot,
            Binding::Keyboard {
                modifiers: 0,
                key: 0x3A,
            },
        );
        assert_eq!(plate_from_key(0x3A, &map), Some(Plate::Six));
    }

    #[test]
    fn dom_codes_match_factory_keys() {
        assert_eq!(hid_from_code("Digit1"), Some(0x1E));
        assert_eq!(hid_from_code("Digit0"), Some(0x27));
        assert_eq!(hid_from_code("Minus"), Some(0x2D));
        assert_eq!(hid_from_code("Equal"), Some(0x2E));
        assert_eq!(hid_from_code("KeyA"), Some(0x04));
        assert_eq!(hid_from_code("F1"), Some(0x3A));
        assert_eq!(hid_from_code("ShiftLeft"), None);
    }
}

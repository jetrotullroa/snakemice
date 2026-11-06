mod model;
mod mouse;
mod protocol;
mod worker;

use std::collections::HashMap;
use std::sync::atomic::Ordering;

use dioxus::prelude::*;

use crate::model::{
    body_for, button_for, hid_from_code, modifier_edge_layout, plate_from_key, Binding, Plate,
    BODY, MODIFIER_KEYS, MOUSE_CHOICES, SIX, TWELVE,
};
use crate::worker::{Command, Update};

const CSS: &str = include_str!("style.css");

fn main() {
    dioxus::LaunchBuilder::new()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("SnakeMice")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1120.0, 780.0))
                        .with_min_inner_size(dioxus::desktop::LogicalSize::new(960.0, 680.0)),
                )
                .with_background_color((20, 17, 14, 255))
                .with_disable_context_menu(true)
                .with_menu(None),
        )
        .launch(app);
}

fn app() -> Element {
    let mut connected = use_signal(|| false);
    let mut transport = use_signal(String::new);
    let mut firmware = use_signal(String::new);
    let mut battery = use_signal(|| None::<u8>);
    let mut charging = use_signal(|| None::<bool>);
    let mut detail = use_signal(|| "Looking for the Naga V3 Pro…".to_string());
    let mut notice = use_signal(String::new);
    let mut buttons = use_signal(HashMap::<u8, Binding>::new);
    let mut poll = use_signal(|| None::<u16>);
    let mut active = use_signal(|| None::<Plate>);
    let mut selected = use_signal(|| None::<u8>);
    let mut draft = use_signal(|| None::<Binding>);
    let capturing = use_signal(|| false);
    let armed = use_signal(|| None::<Plate>);
    let mut dpi_x = use_signal(|| 1600u16);
    let mut dpi_y = use_signal(|| 1600u16);
    let mut dpi_dirty = use_signal(|| false);
    let mut link_xy = use_signal(|| true);

    use_hook(|| crate::worker::ensure());
    use_effect(move || {
        let paused = capturing();
        crate::worker::send(Command::PauseKeys(paused));
    });
    use_hook(move || {
        if crate::worker::pump_started().swap(true, Ordering::SeqCst) {
            return;
        }
        spawn(async move {
            loop {
                for update in crate::worker::drain() {
                    match update {
                        Update::Waiting(text) => {
                            connected.set(false);
                            detail.set(text);
                            notice.set(String::new());
                            active.set(None);
                        }
                        Update::Connected(reading) => {
                            connected.set(true);
                            transport.set(reading.transport);
                            firmware.set(reading.firmware);
                            battery.set(reading.battery);
                            charging.set(reading.charging);
                            poll.set(reading.poll_hz);
                            if !dpi_dirty() {
                                if let Some(x) = reading.dpi_x {
                                    dpi_x.set(x);
                                }
                                if let Some(y) = reading.dpi_y {
                                    dpi_y.set(y);
                                }
                            }
                            let map: HashMap<u8, Binding> = reading.buttons.into_iter().collect();
                            if selected().is_none() {
                                selected.set(Some(TWELVE[0].slot));
                                draft.set(map.get(&TWELVE[0].slot).cloned());
                            }
                            buttons.set(map);
                            if reading.warning.is_empty() {
                                notice.set(
                                    "Side plate and top-of-mouse buttons loaded from the mouse."
                                        .into(),
                                );
                            } else {
                                notice.set(reading.warning);
                            }
                        }
                        Update::Power {
                            battery: level,
                            charging: charge,
                        } => {
                            battery.set(level);
                            charging.set(charge);
                        }
                        Update::Button { slot, binding } => {
                            buttons.write().insert(slot, binding.clone());
                            if selected() == Some(slot) {
                                draft.set(Some(binding));
                            }
                        }
                        Update::Dpi { x, y } => {
                            dpi_x.set(x);
                            dpi_y.set(y);
                            dpi_dirty.set(false);
                            notice.set(format!("DPI is {x} × {y}."));
                        }
                        Update::Poll(hz) => {
                            poll.set(Some(hz));
                            notice.set(format!("Polling rate is {hz} Hz."));
                        }
                        Update::Key(key) => {
                            if capturing() {
                                continue;
                            }
                            if let Some(plate) = plate_from_key(key, &buttons.read()) {
                                active.set(Some(plate));
                            }
                        }
                        Update::Notice(text) => notice.set(text),
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(40)).await;
            }
        });
    });

    let saved = selected().and_then(|slot| buttons.read().get(&slot).cloned());
    let draft_binding = draft().or(saved.clone());
    let dirty = selected().is_some() && draft_binding.is_some() && draft_binding != saved;

    rsx! {
        style { {CSS} }
        div { class: "app",
            header { class: "top",
                div {
                    p { class: "mark", "SnakeMice" }
                    h1 { "Naga V3 Pro" }
                    p { class: "lede",
                        "The 12 side buttons and the 6 side buttons are both stored in the mouse. Snap on the 6-button plate and those six are the ones that fire."
                    }
                }
                div { class: "meta",
                    if connected() {
                        div {
                            strong { "{transport()}" }
                            " · firmware {firmware()}"
                        }
                        if let Some(level) = battery() {
                            div {
                                if charging() == Some(true) {
                                    "{level}% charging"
                                } else {
                                    "{level}%"
                                }
                                span { class: "battery",
                                    span { style: "width: {level}%;" }
                                }
                            }
                        }
                    } else {
                        div { "Mouse not connected" }
                    }
                }
            }

            if !notice().is_empty() {
                div { class: "notice", "{notice}" }
            }

            if !connected() {
                section { class: "empty",
                    h2 { "Waiting for the mouse" }
                    p { "{detail}" }
                    div { class: "row",
                        button {
                            class: "solid",
                            r#type: "button",
                            onclick: move |_| crate::worker::send(Command::Rescan),
                            "Look again"
                        }
                    }
                }
            } else {
                div { class: "controls",
                    label { class: "field",
                        "DPI X"
                        input {
                            r#type: "number",
                            min: "100",
                            max: "50000",
                            value: "{dpi_x}",
                            oninput: move |event| {
                                if let Ok(value) = event.value().parse::<u16>() {
                                    dpi_x.set(value);
                                    dpi_dirty.set(true);
                                    if link_xy() {
                                        dpi_y.set(value);
                                    }
                                }
                            }
                        }
                    }
                    label { class: "field",
                        "DPI Y"
                        input {
                            r#type: "number",
                            min: "100",
                            max: "50000",
                            value: "{dpi_y}",
                            disabled: link_xy(),
                            oninput: move |event| {
                                if let Ok(value) = event.value().parse::<u16>() {
                                    dpi_y.set(value);
                                    dpi_dirty.set(true);
                                }
                            }
                        }
                    }
                    label { class: "check",
                        input {
                            r#type: "checkbox",
                            checked: link_xy(),
                            onchange: move |event| {
                                link_xy.set(event.checked());
                                if event.checked() {
                                    dpi_y.set(dpi_x());
                                    dpi_dirty.set(true);
                                }
                            }
                        }
                        "Same X and Y"
                    }
                    button {
                        class: "solid",
                        r#type: "button",
                        onclick: move |_| {
                            crate::worker::send(Command::Dpi { x: dpi_x(), y: dpi_y() });
                        },
                        "Set DPI"
                    }
                    div { class: "row",
                        span { class: "quiet", "Polling" }
                        for hz in [125u16, 500, 1000] {
                            button {
                                class: if poll() == Some(hz) { "poll on" } else { "poll" },
                                r#type: "button",
                                onclick: move |_| crate::worker::send(Command::Poll(hz)),
                                "{hz} Hz"
                            }
                        }
                    }
                }

                BodyControls {
                    buttons: buttons,
                    selected: selected,
                    draft: draft,
                    capturing: capturing,
                }
                div { class: "workspace",
                    div { class: "pads",
                        PlatePad {
                            plate: Plate::Twelve,
                            specs: TWELVE.to_vec(),
                            live: active() == Some(Plate::Twelve),
                            buttons: buttons,
                            selected: selected,
                            draft: draft,
                            armed: armed,
                            capturing: capturing,
                        }
                        PlatePad {
                            plate: Plate::Six,
                            specs: SIX.to_vec(),
                            live: active() == Some(Plate::Six),
                            buttons: buttons,
                            selected: selected,
                            draft: draft,
                            armed: armed,
                            capturing: capturing,
                        }
                    }
                    Editor {
                        selected: selected,
                        draft: draft,
                        saved: saved,
                        dirty: dirty,
                        capturing: capturing,
                        notice: notice,
                    }
                }
            }
        }
    }
}

#[component]
fn BodyControls(
    buttons: Signal<HashMap<u8, Binding>>,
    mut selected: Signal<Option<u8>>,
    mut draft: Signal<Option<Binding>>,
    mut capturing: Signal<bool>,
) -> Element {
    rsx! {
        section { class: "card body-card",
            header {
                h2 { "Top of mouse (not the side plate)" }
                span { class: "hint", "Saved in the mouse — same as Synapse" }
            }
            p { class: "quiet",
                "These sit on the main top shell: the two quick-access buttons left of left-click, the pair by the scroll wheel, plus the ring-finger strip on the right grip. Only the numbered grid on the left/right thumb plate uses the 12- or 6-button sections below."
            }
            div { class: "grid body-grid",
                for body in BODY.iter() {
                    {
                        let slot = body.slot;
                        let on = selected() == Some(slot);
                        let label = buttons.read().get(&slot).map(Binding::summary).unwrap_or_else(|| "…".into());
                        rsx! {
                            button {
                                class: if on { "cell on" } else { "cell" },
                                r#type: "button",
                                onclick: move |_| {
                                    capturing.set(false);
                                    selected.set(Some(slot));
                                    draft.set(buttons.read().get(&slot).cloned());
                                },
                                span { class: "num", "{body.name}" }
                                span { class: "bind", "{label}" }
                            }
                        }
                    }
                }
            }
            div { class: "row",
                button {
                    class: "solid",
                    r#type: "button",
                    onclick: move |_| {
                        crate::worker::send(Command::ApplyLayout(modifier_edge_layout()));
                    },
                    "Apply: Ctrl + Alt edges, DPI under wheel, Shift on ring"
                }
            }
        }
    }
}

#[component]
fn PlatePad(
    plate: Plate,
    specs: Vec<crate::model::SideButton>,
    live: bool,
    buttons: Signal<HashMap<u8, Binding>>,
    mut selected: Signal<Option<u8>>,
    mut draft: Signal<Option<Binding>>,
    mut armed: Signal<Option<Plate>>,
    mut capturing: Signal<bool>,
) -> Element {
    let cols = plate.columns();
    let card_class = if live { "card live" } else { "card" };
    let blurb = match plate {
        Plate::Twelve => "Labels 1–12. Factory keys are 1 through 9, 0, -, =.",
        Plate::Six => {
            "Labels 1–6. Factory keys are 1 through 6. Used automatically with this plate on."
        }
    };

    rsx! {
        section { class: card_class,
            header {
                h2 { "{plate.label()}" }
                span { class: "hint",
                    if live { "Last press was this plate" } else { "" }
                }
            }
            p { class: "quiet", "{blurb}" }
            div {
                class: "grid",
                style: "grid-template-columns: repeat({cols}, minmax(0, 1fr));",
                for spec in specs.iter().copied() {
                    {
                        let slot = spec.slot;
                        let on = selected() == Some(slot);
                        let binding = buttons.read().get(&slot).cloned();
                        let label = binding.as_ref().map(Binding::summary).unwrap_or_else(|| "…".into());
                        let class = if on { "cell on" } else { "cell" };
                        rsx! {
                            button {
                                class: class,
                                r#type: "button",
                                onclick: move |_| {
                                    capturing.set(false);
                                    armed.set(None);
                                    selected.set(Some(slot));
                                    draft.set(buttons.read().get(&slot).cloned());
                                },
                                span { class: "num", "Button {spec.number}" }
                                span { class: "bind", "{label}" }
                            }
                        }
                    }
                }
            }
            div { class: "row",
                button {
                    class: "ghost",
                    r#type: "button",
                    onclick: move |_| {
                        if armed() == Some(plate) {
                            armed.set(None);
                            let pairs = match plate {
                                Plate::Twelve => TWELVE.iter().map(|b| (b.slot, Binding::factory(b))).collect(),
                                Plate::Six => SIX.iter().map(|b| (b.slot, Binding::factory(b))).collect(),
                            };
                            crate::worker::send(Command::Reset(pairs));
                        } else {
                            armed.set(Some(plate));
                        }
                    },
                    if armed() == Some(plate) { "Click again to reset" } else { "Reset this plate" }
                }
            }
        }
    }
}

#[component]
fn Editor(
    selected: Signal<Option<u8>>,
    mut draft: Signal<Option<Binding>>,
    saved: Option<Binding>,
    dirty: bool,
    mut capturing: Signal<bool>,
    mut notice: Signal<String>,
) -> Element {
    let Some(slot) = selected() else {
        return rsx! {
            aside { class: "editor",
                h2 { "Button" }
                p { class: "quiet", "Choose a button on either plate." }
            }
        };
    };
    let side = button_for(slot);
    let body = body_for(slot);
    if side.is_none() && body.is_none() {
        return rsx! {
            aside { class: "editor",
                h2 { "Button" }
                p { class: "quiet", "Choose a top-of-mouse or side-plate button." }
            }
        };
    }
    let binding = draft().or(saved);
    let Some(binding) = binding else {
        let title = side
            .map(|spec| format!("{} · {}", spec.plate.label(), spec.number))
            .or_else(|| body.map(|b| format!("Top · {}", b.name)))
            .unwrap_or_default();
        return rsx! {
            aside { class: "editor",
                h2 { "{title}" }
                p { class: "quiet", "Still reading this button from the mouse." }
            }
        };
    };

    let kind = match &binding {
        Binding::Mouse(_) => "mouse",
        Binding::Disabled => "off",
        Binding::DpiStageUp | Binding::DpiStageDown => "dpi",
        Binding::Keyboard { .. } => "key",
        Binding::Special(_) | Binding::HyperShift | Binding::ScrollModeCycle => "special",
    };
    let summary = binding.summary();
    let factory = side
        .map(|spec| Binding::factory(spec).summary())
        .or_else(|| body.map(|b| b.factory.summary()))
        .unwrap_or_default();
    let detail = body.map(|b| b.detail).unwrap_or("");
    let title = side
        .map(|spec| format!("{} · {}", spec.plate.label(), spec.number))
        .or_else(|| body.map(|b| format!("Top · {}", b.name)))
        .unwrap_or_default();
    let is_default = side
        .map(|spec| binding.is_factory(spec))
        .or_else(|| body.map(|b| binding.is_body_factory(b)))
        .unwrap_or(false);

    rsx! {
        aside { class: "editor",
            header {
                h2 { "{title}" }
                span { class: "hint", if dirty { "Not saved yet" } else { "Stored in the mouse" } }
            }
            div { class: "summary", "{summary}" }
            if !detail.is_empty() {
                p { class: "quiet", "{detail}" }
            }
            p { class: "quiet", "Factory: {factory}." }
            div { class: "row",
                button {
                    class: if is_default { "choice on" } else { "choice" },
                    r#type: "button",
                    onclick: move |_| {
                        capturing.set(false);
                        if let Some(spec) = button_for(slot) {
                            draft.set(Some(Binding::factory(spec)));
                        } else if let Some(body) = body_for(slot) {
                            draft.set(Some(body.factory.clone()));
                        }
                    },
                    "Default"
                }
                button {
                    class: if kind == "key" && !is_default { "choice on" } else { "choice" },
                    r#type: "button",
                    onclick: move |_| {
                        capturing.set(true);
                        if !matches!(draft(), Some(Binding::Keyboard { .. })) {
                            if let Some(spec) = button_for(slot) {
                                draft.set(Some(Binding::factory(spec)));
                            } else {
                                draft.set(Some(Binding::left_control()));
                            }
                        }
                    },
                    "Keyboard"
                }
                button {
                    class: if kind == "mouse" { "choice on" } else { "choice" },
                    r#type: "button",
                    onclick: move |_| {
                        capturing.set(false);
                        let button = match draft() {
                            Some(Binding::Mouse(button)) => button,
                            _ => 5,
                        };
                        draft.set(Some(Binding::Mouse(button)));
                    },
                    "Mouse"
                }
                button {
                    class: if kind == "off" { "choice on" } else { "choice" },
                    r#type: "button",
                    onclick: move |_| {
                        capturing.set(false);
                        draft.set(Some(Binding::Disabled));
                    },
                    "Off"
                }
            }

            if kind == "key" {
                div { class: "row",
                    for (name, key) in MODIFIER_KEYS {
                        button {
                            class: if matches!(draft(), Some(Binding::Keyboard { modifiers: 0, key: k }) if k == key) { "choice on" } else { "choice" },
                            r#type: "button",
                            onclick: move |_| {
                                capturing.set(false);
                                draft.set(Some(Binding::Keyboard { modifiers: 0, key }));
                            },
                            "{name}"
                        }
                    }
                }
                Modifiers { binding: binding.clone(), draft: draft }
                div {
                    class: "capture",
                    tabindex: "0",
                    onmounted: move |event| async move {
                        if capturing() {
                            let _ = event.set_focus(true).await;
                        }
                    },
                    onkeydown: move |event| {
                        event.prevent_default();
                        let code = event.code().to_string();
                        if is_modifier(&code) {
                            return;
                        }
                        let Some(key) = hid_from_code(&code) else {
                            notice.set("That key can't be stored on the mouse. Try a letter, number, or function key.".into());
                            return;
                        };
                        let mods = event.modifiers();
                        let mut bits = 0u8;
                        if mods.ctrl() { bits |= 0x01; }
                        if mods.shift() { bits |= 0x02; }
                        if mods.alt() { bits |= 0x04; }
                        if mods.meta() { bits |= 0x08; }
                        draft.set(Some(Binding::Keyboard { modifiers: bits, key }));
                        notice.set(format!("Ready to save {}.", Binding::Keyboard { modifiers: bits, key }.summary()));
                    },
                    "Click here and press the key"
                }
            }

            if kind == "mouse" {
                div { class: "row",
                    for (button, name) in MOUSE_CHOICES {
                        button {
                            class: if matches!(&binding, Binding::Mouse(current) if *current == button) { "choice on" } else { "choice" },
                            r#type: "button",
                            onclick: move |_| draft.set(Some(Binding::Mouse(button))),
                            "{name}"
                        }
                    }
                }
            }

            if kind == "dpi" || body.is_some() {
                div { class: "row",
                    button {
                        class: if matches!(binding, Binding::DpiStageUp) { "choice on" } else { "choice" },
                        r#type: "button",
                        onclick: move |_| draft.set(Some(Binding::DpiStageUp)),
                        "DPI stage up"
                    }
                    button {
                        class: if matches!(binding, Binding::DpiStageDown) { "choice on" } else { "choice" },
                        r#type: "button",
                        onclick: move |_| draft.set(Some(Binding::DpiStageDown)),
                        "DPI stage down"
                    }
                }
            }

            if kind == "special" {
                p { class: "quiet", "This is still a Razer function. Pick Default, Keyboard, Mouse, DPI, or Off to replace it." }
            }

            div { class: "row",
                button {
                    class: "solid",
                    r#type: "button",
                    disabled: !dirty,
                    onclick: move |_| {
                        if let Some(binding) = draft() {
                            capturing.set(false);
                            crate::worker::send(Command::Write { slot, binding });
                        }
                    },
                    "Save to mouse"
                }
            }
        }
    }
}

#[component]
fn Modifiers(binding: Binding, mut draft: Signal<Option<Binding>>) -> Element {
    let Binding::Keyboard { modifiers, key } = binding else {
        return rsx! {};
    };
    rsx! {
        div { class: "mods",
            ModChip { label: "⌃ Control", bit: 0x01, modifiers, hid: key, draft }
            ModChip { label: "⌥ Option", bit: 0x04, modifiers, hid: key, draft }
            ModChip { label: "⇧ Shift", bit: 0x02, modifiers, hid: key, draft }
            ModChip { label: "⌘ Command", bit: 0x08, modifiers, hid: key, draft }
        }
    }
}

#[component]
fn ModChip(
    label: &'static str,
    bit: u8,
    modifiers: u8,
    hid: u8,
    mut draft: Signal<Option<Binding>>,
) -> Element {
    let on = modifiers & bit != 0;
    rsx! {
        button {
            class: if on { "choice on" } else { "choice" },
            r#type: "button",
            onclick: move |_| {
                let modifiers = if on { modifiers & !bit } else { modifiers | bit };
                draft.set(Some(Binding::Keyboard { modifiers, key: hid }));
            },
            "{label}"
        }
    }
}

fn is_modifier(code: &str) -> bool {
    matches!(
        code,
        "ShiftLeft"
            | "ShiftRight"
            | "ControlLeft"
            | "ControlRight"
            | "AltLeft"
            | "AltRight"
            | "MetaLeft"
            | "MetaRight"
            | "OSLeft"
            | "OSRight"
    )
}

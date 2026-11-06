//! Background HID owner. The UI thread only sends commands and drains updates.

use std::collections::HashSet;
use std::sync::atomic::AtomicBool;
use std::sync::{mpsc, Mutex, Once};
use std::thread;
use std::time::Duration;

use crate::model::Binding;
use crate::mouse::{Reading, Session};

static START: Once = Once::new();
static CMD: Mutex<Option<mpsc::Sender<Command>>> = Mutex::new(None);
static UPDATES: Mutex<Vec<Update>> = Mutex::new(Vec::new());

pub enum Command {
    Rescan,
    Write { slot: u8, binding: Binding },
    Reset(Vec<(u8, Binding)>),
    ApplyLayout(Vec<(u8, Binding)>),
    Dpi { x: u16, y: u16 },
    Poll(u16),
    PauseKeys(bool),
}

pub enum Update {
    Waiting(String),
    Connected(Reading),
    Power {
        battery: Option<u8>,
        charging: Option<bool>,
    },
    Button {
        slot: u8,
        binding: Binding,
    },
    Dpi {
        x: u16,
        y: u16,
    },
    Poll(u16),
    Key(u8),
    Notice(String),
}

pub fn ensure() {
    START.call_once(|| {
        let (tx, rx) = mpsc::channel();
        *CMD.lock().expect("command channel") = Some(tx);
        thread::Builder::new()
            .name("naga-hid".into())
            .spawn(move || run(rx))
            .expect("hid thread");
    });
}

pub fn send(command: Command) {
    ensure();
    if let Some(tx) = CMD.lock().expect("command channel").as_ref() {
        let _ = tx.send(command);
    }
}

pub fn drain() -> Vec<Update> {
    std::mem::take(&mut *UPDATES.lock().expect("updates"))
}

pub fn pump_started() -> &'static AtomicBool {
    static STARTED: AtomicBool = AtomicBool::new(false);
    &STARTED
}

fn push(update: Update) {
    UPDATES.lock().expect("updates").push(update);
}

fn run(rx: mpsc::Receiver<Command>) {
    let mut session = match Session::new() {
        Ok(session) => session,
        Err(err) => {
            push(Update::Waiting(format!("HID isn't available: {err}")));
            return;
        }
    };
    let mut pause_keys = false;
    let mut held = HashSet::new();
    let mut misses = 0u8;
    let mut last_power = std::time::Instant::now() - Duration::from_secs(30);
    let mut last_detail = String::new();
    scan(&mut session, &mut last_detail);

    loop {
        while let Ok(command) = rx.try_recv() {
            match command {
                Command::Rescan => {
                    session.close();
                    scan(&mut session, &mut last_detail);
                }
                Command::Write { slot, binding } => match session.write_button(slot, &binding) {
                    Ok(stored) => {
                        push(Update::Button {
                            slot,
                            binding: stored,
                        });
                        push(Update::Notice("Saved in the mouse.".into()));
                    }
                    Err(err) => push(Update::Notice(err)),
                },
                Command::Reset(pairs) => {
                    apply_many(&mut session, pairs, "Factory keys are back on this plate.");
                }
                Command::ApplyLayout(pairs) => {
                    apply_many(&mut session, pairs, "Layout saved in the mouse.");
                }
                Command::Dpi { x, y } => match session.set_dpi(x, y) {
                    Ok((x, y)) => push(Update::Dpi { x, y }),
                    Err(err) => push(Update::Notice(err)),
                },
                Command::Poll(hz) => match session.set_poll(hz) {
                    Ok(hz) => push(Update::Poll(hz)),
                    Err(err) => push(Update::Notice(err)),
                },
                Command::PauseKeys(paused) => {
                    pause_keys = paused;
                    held.clear();
                }
            }
        }

        if !session.is_open() {
            thread::sleep(Duration::from_millis(800));
            scan(&mut session, &mut last_detail);
            continue;
        }

        if last_power.elapsed() > Duration::from_secs(4) {
            last_power = std::time::Instant::now();
            match session.poll_power() {
                Ok((battery, charging)) => {
                    misses = 0;
                    push(Update::Power { battery, charging });
                }
                Err(err) => {
                    misses = misses.saturating_add(1);
                    if misses >= 2 {
                        session.close();
                        held.clear();
                        let message = format!("{err}\nLooking again…");
                        last_detail = message.clone();
                        push(Update::Waiting(message));
                    }
                }
            }
        }

        if pause_keys {
            held.clear();
        } else {
            let pressed: HashSet<u8> = session.poll_keys().into_iter().collect();
            for key in pressed.difference(&held) {
                push(Update::Key(*key));
            }
            held = pressed;
        }
        thread::sleep(Duration::from_millis(30));
    }
}

fn apply_many(session: &mut Session, pairs: Vec<(u8, Binding)>, ok_message: &str) {
    let mut failed = 0u8;
    for (slot, binding) in pairs {
        match session.write_button(slot, &binding) {
            Ok(stored) => push(Update::Button {
                slot,
                binding: stored,
            }),
            Err(_) => failed += 1,
        }
    }
    if failed == 0 {
        push(Update::Notice(ok_message.into()));
    } else {
        push(Update::Notice(format!(
            "{failed} button(s) didn't save. Try the cable."
        )));
    }
}

fn scan(session: &mut Session, last_detail: &mut String) {
    match session.scan() {
        Ok(Some(reading)) => {
            last_detail.clear();
            let battery = reading
                .battery
                .map(|level| format!("{level}%"))
                .unwrap_or_else(|| "unknown".into());
            eprintln!(
                "snakemice: connected over {} · firmware {} · {} side buttons · battery {battery}",
                reading.transport,
                reading.firmware,
                reading.buttons.len(),
            );
            if !reading.warning.is_empty() {
                eprintln!("snakemice: {}", reading.warning);
            }
            push(Update::Connected(reading));
        }
        Ok(None) => {
            if session.detail != *last_detail {
                *last_detail = session.detail.clone();
                eprintln!("snakemice: {}", last_detail.replace('\n', " "));
                push(Update::Waiting(last_detail.clone()));
            }
        }
        Err(err) => {
            if err != *last_detail {
                *last_detail = err.clone();
                eprintln!("snakemice: {err}");
                push(Update::Waiting(err));
            }
        }
    }
}

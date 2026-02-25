#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::HashMap,
    sync::mpsc,
    process::Command,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TimerAction {
    Popup,
    Lock,
    Shutdown,
    Reboot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimerInfo {
    id: String,
    action: TimerAction,
    target_time: DateTime<Utc>,
    message: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateTimerRequest {
    action: TimerAction,
    target_time: String,
    message: Option<String>,
}

struct TimerEntry {
    info: TimerInfo,
    cancel_tx: mpsc::Sender<()>,
}

#[derive(Clone, Default)]
struct TimerStore {
    inner: Arc<Mutex<HashMap<String, TimerEntry>>>,
}

#[tauri::command]
fn list_timers(state: State<'_, TimerStore>) -> Result<Vec<TimerInfo>, String> {
    let store = state
        .inner
        .lock()
        .map_err(|_| "Failed to lock timer store".to_string())?;

    let mut timers: Vec<TimerInfo> = store.values().map(|entry| entry.info.clone()).collect();
    timers.sort_by_key(|timer| timer.target_time);

    Ok(timers)
}

#[tauri::command]
fn cancel_timer(id: String, state: State<'_, TimerStore>) -> Result<bool, String> {
    let mut store = state
        .inner
        .lock()
        .map_err(|_| "Failed to lock timer store".to_string())?;

    if let Some(entry) = store.remove(&id) {
        let _ = entry.cancel_tx.send(());
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
fn create_timer(request: CreateTimerRequest, state: State<'_, TimerStore>) -> Result<TimerInfo, String> {
    let target = DateTime::parse_from_rfc3339(&request.target_time)
        .map_err(|_| "Invalid date/time format".to_string())?
        .with_timezone(&Utc);

    let now = Utc::now();
    if target <= now {
        return Err("Selected time must be in the future".to_string());
    }

    if matches!(request.action, TimerAction::Popup)
        && request
            .message
            .as_ref()
            .map(|msg| msg.trim().is_empty())
            .unwrap_or(true)
    {
        return Err("Popup timers require a message".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let info = TimerInfo {
        id: id.clone(),
        action: request.action,
        target_time: target,
        message: request.message.map(|msg| msg.trim().to_string()),
        created_at: now,
    };

    let (cancel_tx, cancel_rx) = mpsc::channel();

    {
        let mut store = state
            .inner
            .lock()
            .map_err(|_| "Failed to lock timer store".to_string())?;

        store.insert(
            id.clone(),
            TimerEntry {
                info: info.clone(),
                cancel_tx,
            },
        );
    }

    let store = state.inner.clone();
    let task_info = info.clone();
    thread::spawn(move || {
        let wait = match (target - Utc::now()).to_std() {
            Ok(duration) => duration,
            Err(_) => Duration::from_secs(0),
        };

        if cancel_rx.recv_timeout(wait).is_err() {
            run_action(&task_info.action, task_info.message.as_deref());
            if let Ok(mut locked) = store.lock() {
                locked.remove(&id);
            }
        }
    });

    Ok(info)
}

fn run_action(action: &TimerAction, message: Option<&str>) {
    match action {
        TimerAction::Popup => {
            if let Some(msg) = message {
                let escaped = msg.replace('"', "\\\"");
                let script = format!(
                    "display dialog \"{}\" with title \"LockPilot\" buttons {{\"OK\"}} default button \"OK\"",
                    escaped
                );
                let _ = run_osascript(&script);
            }
        }
        TimerAction::Lock => {
            // Modern macOS fallback chain for locking:
            // 1) trigger Ctrl+Cmd+Q lock shortcut
            // 2) start screen saver
            // 3) force display sleep
            let locked = run_osascript(
                "tell application \"System Events\" to keystroke \"q\" using {control down, command down}",
            )
            .is_ok()
                || run_osascript("tell application \"System Events\" to start current screen saver")
                    .is_ok();

            if !locked {
                let _ = Command::new("/usr/bin/pmset").arg("displaysleepnow").spawn();
            }
        }
        TimerAction::Shutdown => {
            let _ = run_osascript("tell application \"System Events\" to shut down");
        }
        TimerAction::Reboot => {
            let _ = run_osascript("tell application \"System Events\" to restart");
        }
    }
}

fn run_osascript(script: &str) -> Result<(), String> {
    let output = Command::new("/usr/bin/osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|err| format!("Failed to run osascript: {err}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .manage(TimerStore::default())
        .invoke_handler(tauri::generate_handler![create_timer, list_timers, cancel_timer])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

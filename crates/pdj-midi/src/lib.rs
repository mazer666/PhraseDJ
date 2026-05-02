//! pdj-midi — MIDI device enumeration and event parsing.
//!
//! This crate handles the low-level MIDI I/O using `midir` and converts
//! raw bytes into semantic `MidiEvent`s.

use midir::{MidiInput, MidiInputPort};
use pdj_core::{Error, Result};
use serde::{Deserialize, Serialize};

/// Basic information about a connected MIDI device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiDeviceInfo {
    pub name: String,
    pub port_index: usize,
}

/// List all currently connected MIDI input devices.
pub fn list_devices() -> Result<Vec<MidiDeviceInfo>> {
    let midi_in = match MidiInput::new("PhraseDJ-Input") {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!("MIDI input subsystem unavailable: {}", e);
            return Ok(Vec::new());
        }
    };

    let ports = midi_in.ports();
    let mut devices = Vec::new();

    for (i, port) in ports.iter().enumerate() {
        if let Ok(name) = midi_in.port_name(port) {
            devices.push(MidiDeviceInfo {
                name,
                port_index: i,
            });
        }
    }

    Ok(devices)
}

/// A handle to an open MIDI session.
pub struct MidiSession {
    _conn: midir::MidiInputConnection<()>,
}

impl MidiSession {
    /// Open a connection to a specific MIDI device.
    pub fn open<F>(_name: &str, port: &MidiInputPort, mut callback: F) -> Result<Self>
    where
        F: FnMut(u64, &[u8]) + Send + 'static,
    {
        let midi_in = MidiInput::new("PhraseDJ-Session")
            .map_err(|e| Error::other(format!("Failed to open MIDI session: {}", e)))?;

        let conn = midi_in
            .connect(
                port,
                "phrasedj-port",
                move |stamp, message, _| {
                    callback(stamp, message);
                },
                (),
            )
            .map_err(|e| Error::other(format!("Failed to connect to MIDI port: {}", e)))?;

        Ok(Self { _conn: conn })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_devices_does_not_panic() {
        // In CI there might be no MIDI devices, but it should still return an Ok(empty).
        let res = list_devices();
        assert!(res.is_ok());
    }
}

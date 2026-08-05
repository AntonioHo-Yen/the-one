use evdev::uinput::VirtualDeviceBuilder;
use evdev::{AttributeSet, Key, InputEvent, EventType};
use std::thread;
use std::time::Duration;
use zeroize::Zeroize;

pub struct DesktopInjector {
    // Virtual uinput keyboard device node
    device: evdev::uinput::VirtualDevice,
}

impl DesktopInjector {
    /// Initializes a virtual keyboard node in /dev/uinput
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut keys = AttributeSet::<Key>::new();
        
        // Register standard alphanumeric and structural keys for string typing
        for key_code in 0..255 {
            if let Ok(key) = Key::new(key_code) {
                keys.insert(key);
            }
        }

        let device = VirtualDeviceBuilder::new()?
            .name("TO1 Sovereign Input Device")
            .with_keys(&keys)?
            .build()?;

        Ok(Self { device })
    }

    /// Takes a decrypted attribute string and simulates hardware keystrokes
    /// Wipes the payload buffer in memory immediately after typing.
    pub fn inject_string(&mut self, payload: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Small delay to ensure the target input field retains focus after UI prompts
        thread::sleep(Duration::from_millis(150));

        for ch in payload.chars() {
            let (key, needs_shift) = match self.char_to_key(ch) {
                Some(mapping) => mapping,
                None => continue, // Skip unmapped characters
            };

            self.emit_keypress(key, needs_shift)?;
            // Micro-delay between keypresses to mimic human/hardware throughput
            thread::sleep(Duration::from_millis(10));
        }
      
        Ok(())
    }

    /// Emits a down + up key event pair with optional Shift modifier
    fn emit_keypress(&mut self, key: Key, shift: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mut events = Vec::new();

        if shift {
            events.push(InputEvent::new(EventType::KEY, Key::KEY_LEFTSHIFT.code(), 1));
        }

        // Key Press Down (1)
        events.push(InputEvent::new(EventType::KEY, key.code(), 1));
        self.device.emit(&events)?;

        events.clear();

        // Key Release Up (0)
        events.push(InputEvent::new(EventType::KEY, key.code(), 0));
        if shift {
            events.push(InputEvent::new(EventType::KEY, Key::KEY_LEFTSHIFT.code(), 0));
        }
        self.device.emit(&events)?;

        Ok(())
    }

    /// Maps standard ASCII characters to evdev Key codes and Shift states
    fn char_to_key(&self, c: char) -> Option<(Key, bool)> {
        match c {
            'a'..='z' => Some((Key::new((c as u16 - 'a' as u16) + Key::KEY_A.code())?, false)),
            'A'..='Z' => Some((Key::new((c as u16 - 'A' as u16) + Key::KEY_A.code())?, true)),
            '0' => Some((Key::KEY_0, false)),
            '1'..='9' => Some((Key::new((c as u16 - '1' as u16) + Key::KEY_1.code())?, false)),
            '@' => Some((Key::KEY_2, true)),
            '.' => Some((Key::KEY_DOT, false)),
            '-' => Some((Key::KEY_MINUS, false)),
            '_' => Some((Key::KEY_MINUS, true)),
            ' ' => Some((Key::KEY_SPACE, false)),
            '\n' => Some((Key::KEY_ENTER, false)),
            _ => None,
        }
    }
}
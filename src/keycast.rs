//! The keys a recording presses, grouped into bursts for an on-screen overlay.

use std::time::Duration;

use crate::recording::KeyCast;
use crate::script::{Event, KeyAction, KeySpec, NamedKey};

/// How long a burst stays on screen after its last key, fade included.
pub const VISIBLE_MS: u32 = 1300;
/// A key within this long of the last one joins its burst.
const JOIN_MS: u128 = 900;
/// The overlay keeps the end of a long burst.
const MAX_CHARS: usize = 24;

#[derive(Debug, Default)]
pub struct KeyCaster {
    label: String,
    typed: bool,
    first: Duration,
    last: Option<Duration>,
}

impl KeyCaster {
    /// Records a key the event presses at `t`, recording time.
    pub fn press(&mut self, event: &Event, t: Duration) {
        let (text, typed) = match event {
            Event::Type { text, .. } => (text.clone(), true),
            Event::Key {
                key,
                action: KeyAction::Press,
                ..
            } => (label(key), false),
            _ => return,
        };
        let joins = self
            .last
            .is_some_and(|last| t.saturating_sub(last).as_millis() <= JOIN_MS);
        if joins && typed && self.typed {
            self.label.push_str(&text);
        } else if joins {
            self.label.push(' ');
            self.label.push_str(&text);
        } else {
            self.label = text;
            self.first = t;
        }
        self.typed = typed;
        self.last = Some(t);
        let len = self.label.chars().count();
        if len > MAX_CHARS {
            self.label = self.label.chars().skip(len - MAX_CHARS).collect();
        }
    }

    /// The overlay at `t`, while the last burst is recent.
    pub fn at(&self, t: Duration) -> Option<KeyCast> {
        let last = self.last?;
        let idle_ms = t.checked_sub(last)?.as_millis() as u32;
        (idle_ms < VISIBLE_MS).then(|| KeyCast {
            label: self.label.trim().to_string(),
            shown_ms: t.saturating_sub(self.first).as_millis() as u32,
            idle_ms,
        })
    }
}

/// A key as macOS writes it in menus: modifier symbols, then the key.
fn label(key: &KeySpec) -> String {
    let mut s = String::new();
    if key.mods.ctrl {
        s.push('⌃');
    }
    if key.mods.alt {
        s.push('⌥');
    }
    if key.mods.shift {
        s.push('⇧');
    }
    if key.mods.super_key {
        s.push('⌘');
    }
    let name = match key.key {
        NamedKey::Enter => "↵".to_string(),
        NamedKey::Escape => "esc".to_string(),
        NamedKey::Tab => "⇥".to_string(),
        NamedKey::Backspace => "⌫".to_string(),
        NamedKey::Delete => "⌦".to_string(),
        NamedKey::Insert => "ins".to_string(),
        NamedKey::Space => "␣".to_string(),
        NamedKey::Up => "↑".to_string(),
        NamedKey::Down => "↓".to_string(),
        NamedKey::Left => "←".to_string(),
        NamedKey::Right => "→".to_string(),
        NamedKey::PageUp => "⇞".to_string(),
        NamedKey::PageDown => "⇟".to_string(),
        NamedKey::Home => "↖".to_string(),
        NamedKey::End => "↘".to_string(),
        NamedKey::Shift => "⇧".to_string(),
        NamedKey::Control => "⌃".to_string(),
        NamedKey::Alt => "⌥".to_string(),
        NamedKey::Super => "⌘".to_string(),
        NamedKey::ScrollUp | NamedKey::ScrollDown => return s,
        NamedKey::Char(c) if key.mods.any() => c.to_uppercase().collect(),
        NamedKey::Char(c) => c.to_string(),
    };
    s.push_str(&name);
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::ModSet;

    fn key(k: NamedKey, ctrl: bool) -> Event {
        Event::Key {
            key: KeySpec {
                key: k,
                mods: ModSet {
                    ctrl,
                    ..ModSet::NONE
                },
            },
            action: KeyAction::Press,
            count: 1,
            delay: Duration::ZERO,
        }
    }

    fn typed(s: &str) -> Event {
        Event::Type {
            text: s.to_string(),
            delay: Duration::ZERO,
        }
    }

    #[test]
    fn typed_letters_form_a_word_and_keys_a_sequence() {
        let mut k = KeyCaster::default();
        let ms = Duration::from_millis;
        for (i, c) in "rawr".chars().enumerate() {
            k.press(&typed(&c.to_string()), ms(i as u64 * 70));
        }
        k.press(&key(NamedKey::Enter, false), ms(400));
        assert_eq!(k.at(ms(500)).unwrap().label, "rawr ↵");
        k.press(&key(NamedKey::Char('f'), true), ms(5000));
        let cast = k.at(ms(5100)).unwrap();
        assert_eq!(cast.label, "⌃F");
        assert_eq!(cast.shown_ms, 100);
        assert!(k.at(ms(5000 + VISIBLE_MS as u64)).is_none());
    }
}

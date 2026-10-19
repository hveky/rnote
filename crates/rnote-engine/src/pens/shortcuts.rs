// Imports
use super::PenStyle;
use rnote_compose::builders::ShapeBuilderType;
use rnote_compose::penevent::{KeyBinding, ShortcutKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[repr(u32)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    num_derive::FromPrimitive,
    num_derive::ToPrimitive,
)]
#[serde(rename = "shortcut_mode")]
pub enum ShortcutMode {
    #[serde(rename = "temporary")]
    Temporary,
    #[serde(rename = "permanent")]
    Permanent,
    #[serde(rename = "toggle")]
    Toggle,
    #[serde(rename = "disabled")]
    Disabled,
}

impl TryFrom<u32> for ShortcutMode {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u32(value).ok_or_else(|| {
            anyhow::anyhow!("ShortcutMode try_from::<u32>() for value {} failed", value)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename = "shortcut_action")]
pub enum ShortcutAction {
    #[serde(rename = "change_pen_style")]
    ChangePenStyle {
        #[serde(rename = "style")]
        style: PenStyle,
        #[serde(rename = "mode")]
        mode: ShortcutMode,
    },
    /// Switches the pen style, and optionally also the shaper's target shape
    /// builder (used by the Excalidraw-style shape shortcuts).
    #[serde(rename = "switch_pen_style")]
    SwitchPenStyle {
        #[serde(rename = "style")]
        style: PenStyle,
        #[serde(rename = "shape_builder")]
        shape_builder: Option<ShapeBuilderType>,
        #[serde(rename = "mode")]
        mode: ShortcutMode,
    },
}

/// The registered shortcut actions for the given shortcut keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "shortcuts")]
pub struct Shortcuts {
    #[serde(flatten)]
    shortcuts: HashMap<ShortcutKey, ShortcutAction>,
    /// True once the Excalidraw-style defaults were seeded into this config.
    /// Lives inside the config (not a separate setting) so that it is saved
    /// atomically together with the shortcuts themselves.
    #[serde(rename = "defaults_seeded", default)]
    defaults_seeded: bool,
}

impl Default for Shortcuts {
    fn default() -> Self {
        let mut shortcuts = HashMap::<ShortcutKey, ShortcutAction>::default();
        shortcuts.insert(
            ShortcutKey::StylusPrimaryButton,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Eraser,
                mode: ShortcutMode::Temporary,
            },
        );
        shortcuts.insert(
            ShortcutKey::StylusSecondaryButton,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Selector,
                mode: ShortcutMode::Temporary,
            },
        );
        shortcuts.insert(
            ShortcutKey::MouseSecondaryButton,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Shaper,
                mode: ShortcutMode::Temporary,
            },
        );
        shortcuts.insert(
            ShortcutKey::TouchTwoFingerLongPress,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Eraser,
                mode: ShortcutMode::Toggle,
            },
        );
        shortcuts.insert(
            ShortcutKey::KeyboardCtrlSpace,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Tools,
                mode: ShortcutMode::Toggle,
            },
        );
        shortcuts.insert(
            ShortcutKey::DrawingPadButton0,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Brush,
                mode: ShortcutMode::Permanent,
            },
        );
        shortcuts.insert(
            ShortcutKey::DrawingPadButton1,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Shaper,
                mode: ShortcutMode::Permanent,
            },
        );
        shortcuts.insert(
            ShortcutKey::DrawingPadButton2,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Typewriter,
                mode: ShortcutMode::Permanent,
            },
        );
        shortcuts.insert(
            ShortcutKey::DrawingPadButton3,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Eraser,
                mode: ShortcutMode::Permanent,
            },
        );

        for (binding, action) in excalidraw_default_shortcuts() {
            shortcuts.insert(ShortcutKey::KeyboardCustom(binding), action);
        }

        Self {
            shortcuts,
            defaults_seeded: true,
        }
    }
}

/// Lowercase letter keyvals for the Excalidraw-style default shortcuts.
const KEYVAL_V: u32 = 0x76;
const KEYVAL_R: u32 = 0x72;
const KEYVAL_O: u32 = 0x6f;
const KEYVAL_A: u32 = 0x61;
const KEYVAL_L: u32 = 0x6c;
const KEYVAL_T: u32 = 0x74;
const KEYVAL_E: u32 = 0x65;

/// The Excalidraw-style letter shortcuts: V selector, R rectangle, O ellipse,
/// A arrow, L line, T typewriter, E eraser. No modifiers.
fn excalidraw_default_shortcuts() -> Vec<(KeyBinding, ShortcutAction)> {
    fn key(keyval: u32) -> KeyBinding {
        KeyBinding {
            keyval,
            modifiers: 0,
        }
    }
    let switch = |style, shape_builder| ShortcutAction::SwitchPenStyle {
        style,
        shape_builder,
        mode: ShortcutMode::Permanent,
    };
    let style = |style| ShortcutAction::ChangePenStyle {
        style,
        mode: ShortcutMode::Permanent,
    };

    vec![
        (key(KEYVAL_V), style(PenStyle::Selector)),
        (key(KEYVAL_R), switch(PenStyle::Shaper, Some(ShapeBuilderType::Rectangle))),
        (key(KEYVAL_O), switch(PenStyle::Shaper, Some(ShapeBuilderType::Ellipse))),
        (key(KEYVAL_A), switch(PenStyle::Shaper, Some(ShapeBuilderType::Arrow))),
        (key(KEYVAL_L), switch(PenStyle::Shaper, Some(ShapeBuilderType::Line))),
        (key(KEYVAL_T), style(PenStyle::Typewriter)),
        (key(KEYVAL_E), style(PenStyle::Eraser)),
    ]
}

impl Deref for Shortcuts {
    type Target = HashMap<ShortcutKey, ShortcutAction>;

    fn deref(&self) -> &Self::Target {
        &self.shortcuts
    }
}

impl DerefMut for Shortcuts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.shortcuts
    }
}

/// GDK_KEY_space.
pub const LEGACY_CTRL_SPACE_KEYVAL: u32 = 0x20;
/// GdkModifierType CONTROL_MASK.
pub const LEGACY_CTRL_SPACE_MODIFIERS: u32 = 0x4;

impl Shortcuts {
    /// Moves a legacy `KeyboardCtrlSpace` entry into a
    /// `KeyboardCustom(KeyBinding { ctrl + space })` entry, preserving its
    /// action. Idempotent. Called once after deserializing the engine config.
    pub fn migrate_legacy_keyboard_shortcut(&mut self) {
        if let Some(action) = self.shortcuts.remove(&ShortcutKey::KeyboardCtrlSpace) {
            self.shortcuts
                .entry(ShortcutKey::KeyboardCustom(KeyBinding {
                    keyval: LEGACY_CTRL_SPACE_KEYVAL,
                    modifiers: LEGACY_CTRL_SPACE_MODIFIERS,
                }))
                .or_insert(action);
        }
    }

    /// Seeds the Excalidraw-style letter shortcuts into an existing config,
    /// unless they were already seeded before (`defaults_seeded` flag). Only
    /// inserts a binding whose keyval/modifiers combination is not already
    /// registered, so a user's own binding for the same key wins. The flag is
    /// part of the serialized config, so seeding and its marker are saved
    /// atomically together — a seed deleted after seeding stays deleted.
    pub fn seed_missing_excalidraw_defaults(&mut self) {
        if self.defaults_seeded {
            return;
        }
        for (binding, action) in excalidraw_default_shortcuts() {
            let key = ShortcutKey::KeyboardCustom(binding);
            self.shortcuts.entry(key).or_insert(action);
        }
        self.defaults_seeded = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcuts_default_roundtrip() {
        let shortcuts = Shortcuts::default();
        let json = serde_json::to_string(&shortcuts).unwrap();
        let deserialized: Shortcuts = serde_json::from_str(&json).unwrap();
        assert_eq!(*deserialized, *shortcuts);
    }

    #[test]
    fn legacy_ctrl_space_migrates_to_keyboard_custom() {
        // Old configs serialize every ShortcutKey as a bare string.
        let legacy_json = r#"{"keyboard_ctrl_space":{"change_pen_style":{"style":"typewriter","mode":"toggle"}},"stylus_primary_button":{"change_pen_style":{"style":"eraser","mode":"temporary"}}}"#;
        let mut shortcuts: Shortcuts = serde_json::from_str(legacy_json).unwrap();
        assert!(shortcuts.contains_key(&ShortcutKey::KeyboardCtrlSpace));

        shortcuts.migrate_legacy_keyboard_shortcut();

        assert!(!shortcuts.contains_key(&ShortcutKey::KeyboardCtrlSpace));
        let action = shortcuts
            .get(&ShortcutKey::KeyboardCustom(KeyBinding {
                keyval: LEGACY_CTRL_SPACE_KEYVAL,
                modifiers: LEGACY_CTRL_SPACE_MODIFIERS,
            }))
            .unwrap();
        assert!(matches!(
            action,
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Typewriter,
                mode: ShortcutMode::Toggle,
            }
        ));
        // The other entries survive untouched.
        assert!(matches!(
            shortcuts.get(&ShortcutKey::StylusPrimaryButton),
            Some(ShortcutAction::ChangePenStyle {
                style: PenStyle::Eraser,
                ..
            })
        ));

        // Second call is a no-op.
        let snapshot = serde_json::to_string(&*shortcuts).unwrap();
        shortcuts.migrate_legacy_keyboard_shortcut();
        assert_eq!(serde_json::to_string(&*shortcuts).unwrap(), snapshot);
    }

    #[test]
    fn keyboard_custom_serializes_as_string_key() {
        let mut shortcuts = Shortcuts::default();
        shortcuts.insert(
            ShortcutKey::KeyboardCustom(KeyBinding {
                keyval: 0x65,         // 'e'
                modifiers: 0x4 | 0x1, // Ctrl + Shift
            }),
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Shaper,
                mode: ShortcutMode::Permanent,
            },
        );
        let json = serde_json::to_string(&shortcuts).unwrap();
        assert!(
            json.contains(r#""keyboard_custom:101:5""#),
            "unexpected serialization: {json}"
        );

        let deserialized: Shortcuts = serde_json::from_str(&json).unwrap();
        assert_eq!(*deserialized, *shortcuts);
    }

    #[test]
    fn switch_pen_style_roundtrip() {
        let mut shortcuts = Shortcuts::default();
        shortcuts.insert(
            ShortcutKey::KeyboardCustom(KeyBinding {
                keyval: 0x72, // 'r'
                modifiers: 0,
            }),
            ShortcutAction::SwitchPenStyle {
                style: PenStyle::Shaper,
                shape_builder: Some(rnote_compose::builders::ShapeBuilderType::Rectangle),
                mode: ShortcutMode::Permanent,
            },
        );
        let json = serde_json::to_string(&shortcuts).unwrap();
        assert!(
            json.contains(
                r#""switch_pen_style":{"style":"shaper","shape_builder":"rectangle","mode":"permanent"}"#
            ),
            "unexpected serialization: {json}"
        );

        let deserialized: Shortcuts = serde_json::from_str(&json).unwrap();
        assert_eq!(*deserialized, *shortcuts);
    }

    #[test]
    fn legacy_change_pen_style_json_still_parses() {
        let legacy_json = r#"{"keyboard_custom:118:0":{"change_pen_style":{"style":"selector","mode":"permanent"}}}"#;
        let shortcuts: Shortcuts = serde_json::from_str(legacy_json).unwrap();
        assert!(matches!(
            shortcuts.get(&ShortcutKey::KeyboardCustom(KeyBinding {
                keyval: 0x76,
                modifiers: 0,
            })),
            Some(ShortcutAction::ChangePenStyle {
                style: PenStyle::Selector,
                ..
            })
        ));
    }

    #[test]
    fn excalidraw_defaults_seeded_only_when_missing() {
        let v_binding = KeyBinding {
            keyval: 0x76,
            modifiers: 0,
        };

        // An unseeded config receives all 7 presets.
        let mut shortcuts = Shortcuts {
            shortcuts: HashMap::default(),
            defaults_seeded: false,
        };
        shortcuts.seed_missing_excalidraw_defaults();
        assert_eq!(shortcuts.len(), 7);
        assert!(matches!(
            shortcuts.get(&ShortcutKey::KeyboardCustom(v_binding)),
            Some(ShortcutAction::ChangePenStyle {
                style: PenStyle::Selector,
                ..
            })
        ));

        // A user's own binding for the same key is preserved.
        let mut shortcuts = Shortcuts {
            shortcuts: HashMap::default(),
            defaults_seeded: false,
        };
        shortcuts.insert(
            ShortcutKey::KeyboardCustom(v_binding),
            ShortcutAction::ChangePenStyle {
                style: PenStyle::Tools,
                mode: ShortcutMode::Toggle,
            },
        );
        shortcuts.seed_missing_excalidraw_defaults();
        assert_eq!(shortcuts.len(), 7);
        assert!(matches!(
            shortcuts.get(&ShortcutKey::KeyboardCustom(v_binding)),
            Some(ShortcutAction::ChangePenStyle {
                style: PenStyle::Tools,
                ..
            })
        ));

        // Second call is a no-op (the flag flipped).
        let snapshot = serde_json::to_string(&*shortcuts).unwrap();
        shortcuts.seed_missing_excalidraw_defaults();
        assert_eq!(serde_json::to_string(&*shortcuts).unwrap(), snapshot);

        // A seeded config that had a preset deleted stays without it.
        let mut shortcuts = Shortcuts {
            shortcuts: HashMap::default(),
            defaults_seeded: true,
        };
        shortcuts.seed_missing_excalidraw_defaults();
        assert!(shortcuts.is_empty());
    }
}

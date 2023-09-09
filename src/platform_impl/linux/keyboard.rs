use gdk::{
    keys::{constants::*, Key},
    EventKey, ModifierType,
};

use crate::event::{ModifiersState, VirtualKeyCode};

const MODIFIER_MAP: &[(ModifierType, ModifiersState)] = &[
    (ModifierType::SHIFT_MASK, ModifiersState::SHIFT),
    (ModifierType::MOD1_MASK, ModifiersState::ALT),
    (ModifierType::CONTROL_MASK, ModifiersState::CTRL),
    (ModifierType::SUPER_MASK, ModifiersState::LOGO),
];

// we use the EventKey to extract the modifier mainly because
// we need to have the modifier before the second key is entered to follow
// other os' logic -- this way we can emit the new `ModifiersState` before
// we receive the next key, if needed the developer can update his local state.
pub(crate) fn get_modifiers(key: &EventKey) -> ModifiersState {
    let state = key.state();

    // start with empty state
    let mut result = ModifiersState::empty();

    // loop trough our modifier map
    for (gdk_mod, modifier) in MODIFIER_MAP {
        if state == *gdk_mod {
            result |= *modifier;
        }
    }
    result
}

#[allow(clippy::just_underscores_and_digits, non_upper_case_globals)]
pub(crate) fn gdk_key_to_virtual_key(gdk_key: Key) -> Option<VirtualKeyCode> {
    match gdk_key {
        Escape => Some(VirtualKeyCode::Escape),
        BackSpace => Some(VirtualKeyCode::Backslash),
        Tab | ISO_Left_Tab => Some(VirtualKeyCode::Tab),
        Return => Some(VirtualKeyCode::Return),
        Control_L => Some(VirtualKeyCode::LControl),
        Control_R => Some(VirtualKeyCode::RControl),
        Alt_L => Some(VirtualKeyCode::LAlt),
        Alt_R => Some(VirtualKeyCode::RAlt),
        Shift_L => Some(VirtualKeyCode::LShift),
        Shift_R => Some(VirtualKeyCode::RShift),
        // TODO: investigate mapping. Map Meta_[LR]?
        Super_L => Some(VirtualKeyCode::LWin),
        Super_R => Some(VirtualKeyCode::RWin),
        Caps_Lock => Some(VirtualKeyCode::Capital),
        F1 => Some(VirtualKeyCode::F1),
        F2 => Some(VirtualKeyCode::F2),
        F3 => Some(VirtualKeyCode::F3),
        F4 => Some(VirtualKeyCode::F4),
        F5 => Some(VirtualKeyCode::F5),
        F6 => Some(VirtualKeyCode::F6),
        F7 => Some(VirtualKeyCode::F7),
        F8 => Some(VirtualKeyCode::F8),
        F9 => Some(VirtualKeyCode::F9),
        F10 => Some(VirtualKeyCode::F10),
        F11 => Some(VirtualKeyCode::F11),
        F12 => Some(VirtualKeyCode::F12),

        Print => Some(VirtualKeyCode::Snapshot),
        Scroll_Lock => Some(VirtualKeyCode::Scroll),
        // Pause/Break not audio.
        Pause => Some(VirtualKeyCode::Pause),

        Insert => Some(VirtualKeyCode::Insert),
        Delete => Some(VirtualKeyCode::Delete),
        Home => Some(VirtualKeyCode::Home),
        End => Some(VirtualKeyCode::End),
        Page_Up => Some(VirtualKeyCode::PageUp),
        Page_Down => Some(VirtualKeyCode::PageDown),
        Num_Lock => Some(VirtualKeyCode::Numlock),

        Up => Some(VirtualKeyCode::Up),
        Down => Some(VirtualKeyCode::Down),
        Left => Some(VirtualKeyCode::Left),
        Right => Some(VirtualKeyCode::Right),
        // Clear => Some(VirtualKeyCode::Clear),

        // Menu => Some(VirtualKeyCode::ContextMenu),
        // WakeUp => Some(VirtualKeyCode::WakeUp),
        // Launch0 => Some(VirtualKeyCode::LaunchApplication1),
        // Launch1 => Some(VirtualKeyCode::LaunchApplication2),
        // ISO_Level3_Shift => Some(VirtualKeyCode::AltGraph),

        // KP_Begin => Some(VirtualKeyCode::Clear),
        // KP_Delete => Some(VirtualKeyCode::Delete),
        // KP_Down => Some(VirtualKeyCode::ArrowDown),
        // KP_End => Some(VirtualKeyCode::End),
        // KP_Enter => Some(VirtualKeyCode::NumpadEnter),
        // KP_F1 => Some(VirtualKeyCode::F1),
        // KP_F2 => Some(VirtualKeyCode::F2),
        // KP_F3 => Some(VirtualKeyCode::F3),
        // KP_F4 => Some(VirtualKeyCode::F4),
        // KP_Home => Some(VirtualKeyCode::Home),
        // KP_Insert => Some(VirtualKeyCode::Insert),
        // KP_Left => Some(VirtualKeyCode::ArrowLeft),
        // KP_Page_Down => Some(VirtualKeyCode::PageDown),
        // KP_Page_Up => Some(VirtualKeyCode::PageUp),
        // KP_Right => Some(VirtualKeyCode::ArrowRight),
        // // KP_Separator? What does it map to?
        // KP_Tab => Some(VirtualKeyCode::Tab),
        // KP_Up => Some(VirtualKeyCode::ArrowUp),
        // TODO: more mappings (media etc)
        _ => None,
    }
}

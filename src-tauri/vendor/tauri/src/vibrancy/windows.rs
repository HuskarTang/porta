// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]

use std::ffi::c_void;

use crate::utils::config::WindowEffectsConfig;
use crate::window::{Color, Effect};
use raw_window_handle::HasWindowHandle;
use windows::Win32::Foundation::HWND;

pub fn apply_effects(window: impl HasWindowHandle, effects: WindowEffectsConfig) {
  // window-vibrancy has been removed from Porta's vendored tauri.
  // Porta does not rely on window effects, so this is intentionally a no-op.
  let _ = window;
  let _ = effects;
}

pub fn clear_effects(window: impl HasWindowHandle) {
  // No-op (window-vibrancy removed).
  let _ = window;
}

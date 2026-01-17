// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// window-vibrancy has been removed from Porta's vendored tauri to avoid startup panics on macOS 15.6.
// Porta does not rely on window effects, so this implementation is intentionally a no-op.

use crate::utils::config::WindowEffectsConfig;
use raw_window_handle::HasWindowHandle;

pub fn apply_effects(_window: impl HasWindowHandle, _effects: WindowEffectsConfig) {}

// Feature #10: Display Config — Options menu overlay for resolution switching
// and fullscreen toggle.
//
// Design Reference: docs/features/10-display-config.md §4, §6, §8
// SRS Reference: FR-017
//
// Provides OptionsMenuState (menu state machine) and DisplayAction (IAPI-011 payload).
// Consumer of IAPI-011: GameLoop::apply_display(w, h, fullscreen) (Provider: F01 Engine).

// ============================================================================
// Constants (§8 Data Model — inline with design doc values)
// ============================================================================

/// Supported display resolutions in display order.
const RESOLUTIONS: [(u32, u32); 3] = [
    (1280, 720),
    (1920, 1080),
    (2560, 1440),
];

/// Menu item count (3 resolutions + 1 fullscreen toggle row).
const MENU_ITEM_COUNT: usize = 4;

/// Index of the fullscreen toggle row (last menu item).
const FULLSCREEN_INDEX: usize = MENU_ITEM_COUNT - 1;

/// Color constants from §Visual Rendering Contract.
const GOLD_R: u8 = 248;
const GOLD_G: u8 = 184;
const GOLD_B: u8 = 0;
const WHITE_R: u8 = 255;
const WHITE_G: u8 = 255;
const WHITE_B: u8 = 255;

/// Background overlay color: dark semi-transparent (#1A1A2E at 85% alpha).
const BG_R: f32 = 0.10;
const BG_G: f32 = 0.10;
const BG_B: f32 = 0.18;
const BG_A: f32 = 0.85;

/// Font sizes per §Visual Rendering Contract.
const TITLE_FONT_SIZE: u16 = 12;
const HINT_FONT_SIZE: u16 = 8;

/// Menu layout positions on the 480x270 virtual canvas.
const TITLE_Y: f32 = 50.0;
const ITEM_START_Y: f32 = 90.0;
const ITEM_SPACING: f32 = 30.0;
const HINT_Y: f32 = 240.0;
const CENTER_X: f32 = 240.0;

// ============================================================================
// DisplayAction (§8 Data Model)
// ============================================================================

/// Carries the resolved display settings from menu confirmation to IAPI-011.
///
/// Fields (§8):
/// - `resolution`: the selected or saved windowed resolution (w, h).
/// - `fullscreen`: whether fullscreen mode is active.
///
/// This struct's fields align one-to-one with `GameLoop::apply_display(w, h, fullscreen)`
/// parameters (IAPI-011).
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayAction {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
}

// ============================================================================
// OptionsMenuState (§4 Interface Contract, §8 Data Model)
// ============================================================================

/// Options menu overlay state machine.
///
/// Manages menu navigation (ArrowUp/Down with wrap-around), fullscreen toggle,
/// and resolution selection. Produces `DisplayAction` on confirmation for
/// `GameLoop::apply_display()` (IAPI-011).
///
/// Fields (§8):
/// - `selected_index`: menu cursor position (0-2 = resolution rows, 3 = fullscreen toggle row).
/// - `resolutions`: hardcoded supported resolution tuples.
/// - `fullscreen`: current fullscreen state (toggled in menu).
/// - `previous_window_resolution`: saved windowed resolution for fullscreen exit restore.
pub struct OptionsMenuState {
    pub selected_index: usize,
    pub resolutions: [(u32, u32); 3],
    pub fullscreen: bool,
    pub previous_window_resolution: (u32, u32),
}

impl OptionsMenuState {
    /// Create a new OptionsMenuState initialized from current display settings.
    ///
    /// # Preconditions (§4)
    /// - `(current_w, current_h)` should be a supported resolution.
    ///
    /// # Postconditions (§4)
    /// - `selected_index` points to the index of `(current_w, current_h)` in `resolutions`.
    /// - If `(current_w, current_h)` is unsupported, falls back to index 0 (1280x720).
    /// - `fullscreen = current_fs`.
    /// - `previous_window_resolution = (current_w, current_h)`.
    pub fn new(current_w: u32, current_h: u32, current_fs: bool) -> Self {
        let selected_index = RESOLUTIONS
            .iter()
            .position(|&(rw, rh)| rw == current_w && rh == current_h)
            .unwrap_or(0);

        Self {
            selected_index,
            resolutions: RESOLUTIONS,
            fullscreen: current_fs,
            previous_window_resolution: (current_w, current_h),
        }
    }

    /// Move the selection cursor up by one position, with wrap-around.
    ///
    /// # Postconditions (§4)
    /// - If `selected_index == 0`, wraps to 3 (fullscreen toggle row).
    /// - Otherwise, `selected_index` decrements by 1.
    /// - `selected_index` always remains in [0, 3].
    pub fn navigate_up(&mut self) {
        if self.selected_index == 0 {
            self.selected_index = FULLSCREEN_INDEX;
        } else {
            self.selected_index -= 1;
        }
    }

    /// Move the selection cursor down by one position, with wrap-around.
    ///
    /// # Postconditions (§4)
    /// - If `selected_index == 3`, wraps to 0 (first resolution row).
    /// - Otherwise, `selected_index` increments by 1.
    /// - `selected_index` always remains in [0, 3].
    pub fn navigate_down(&mut self) {
        if self.selected_index == FULLSCREEN_INDEX {
            self.selected_index = 0;
        } else {
            self.selected_index += 1;
        }
    }

    /// Toggle the fullscreen flag on or off.
    ///
    /// # Preconditions (§4)
    /// - `selected_index == 3` (focus on the fullscreen toggle row).
    ///
    /// # Postconditions (§4)
    /// - If precondition met: `fullscreen` is flipped (`true → false` or `false → true`).
    /// - If precondition NOT met: silent no-op (design §4 specifies this) —
    ///   operand is silently ignored when focus is not on the toggle row.
    pub fn toggle_fullscreen(&mut self) {
        // Guard: only effective when focus is on the fullscreen toggle row (§4).
        if self.selected_index == FULLSCREEN_INDEX {
            self.fullscreen = !self.fullscreen;
        }
    }

    /// Produce a `DisplayAction` from the current menu state.
    ///
    /// # Postconditions (§4)
    /// - If `selected_index < 3` (resolution row): returns `DisplayAction` with
    ///   `resolution` from the selected resolution row and `fullscreen = self.fullscreen`.
    /// - If `selected_index == 3` (fullscreen row): returns `DisplayAction` with
    ///   `resolution = self.previous_window_resolution` (saved windowed resolution)
    ///   and `fullscreen = self.fullscreen`.
    pub fn confirm_action(&self) -> DisplayAction {
        let resolution = if self.selected_index < 3 {
            self.resolutions[self.selected_index]
        } else {
            self.previous_window_resolution
        };

        DisplayAction {
            resolution,
            fullscreen: self.fullscreen,
        }
    }

    /// Return the resolution tuple at the current cursor position.
    ///
    /// # Postconditions (§4)
    /// - Returns `self.resolutions[self.selected_index]`.
    /// - Caller must ensure `selected_index < 3` for meaningful resolution data.
    pub fn selected_resolution(&self) -> (u32, u32) {
        self.resolutions[self.selected_index]
    }

    // -----------------------------------------------------------------------
    // Render-driving associated functions (§Visual Rendering Contract)
    // -----------------------------------------------------------------------

    /// Background overlay color: dark semi-transparent (#1A1A2E, alpha 0.85).
    pub fn background_color() -> (f32, f32, f32, f32) {
        (BG_R, BG_G, BG_B, BG_A)
    }

    /// Background overlay rectangle covering the full virtual canvas.
    pub fn background_rect(vp_w: f32, vp_h: f32) -> (f32, f32, f32, f32) {
        (0.0, 0.0, vp_w, vp_h)
    }

    /// Title text rendered at the top of the menu panel.
    pub fn title_text() -> &'static str {
        "OPTIONS"
    }

    /// Title font size in pixels.
    pub fn title_font_size() -> u16 {
        TITLE_FONT_SIZE
    }

    /// Title position (x, y) — centered horizontally, near top.
    pub fn title_pos(_vp_w: f32, _vp_h: f32) -> (f32, f32) {
        (CENTER_X, TITLE_Y)
    }

    /// Normal (non-highlighted) text color: white.
    pub fn normal_text_color() -> (u8, u8, u8) {
        (WHITE_R, WHITE_G, WHITE_B)
    }

    /// Resolution label for the given menu item index.
    ///
    /// Returns the human-readable label ("720p", "1080p", "1440p").
    pub fn resolution_label(index: usize) -> &'static str {
        match index {
            0 => "720p",
            1 => "1080p",
            2 => "1440p",
            _ => "",
        }
    }

    /// Item text color — gold if selected, white otherwise.
    pub fn item_color(index: usize, selected_index: usize) -> (u8, u8, u8) {
        if index == selected_index {
            (GOLD_R, GOLD_G, GOLD_B)
        } else {
            (WHITE_R, WHITE_G, WHITE_B)
        }
    }

    /// Item position (x, y) for the given menu item index.
    ///
    /// Items are vertically spaced by ITEM_SPACING starting from ITEM_START_Y.
    /// Centered horizontally on the viewport.
    pub fn item_pos(index: usize, _vp_w: f32, _vp_h: f32) -> (f32, f32) {
        let y = ITEM_START_Y + (index as f32) * ITEM_SPACING;
        (CENTER_X, y)
    }

    /// Fullscreen toggle label text, reflecting the current fullscreen state.
    pub fn fullscreen_label(fullscreen: bool) -> &'static str {
        if fullscreen {
            "FULLSCREEN: ON"
        } else {
            "FULLSCREEN: OFF"
        }
    }

    /// Hint text shown at the bottom of the menu panel.
    pub fn hint_text() -> &'static str {
        "Press ESC to close"
    }

    /// Hint font size in pixels.
    pub fn hint_font_size() -> u16 {
        HINT_FONT_SIZE
    }

    /// Hint position (x, y) — centered horizontally, near bottom.
    pub fn hint_pos(_vp_w: f32, _vp_h: f32) -> (f32, f32) {
        (CENTER_X, HINT_Y)
    }

    /// Check if a menu item index is currently selected (highlighted).
    pub fn is_item_selected(index: usize, selected_index: usize) -> bool {
        index == selected_index
    }
}

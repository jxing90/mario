// Keyboard input mapping (IAPI-001)
//
// Feature #3: Player Controller — InputState is defined here as the first consumer
// of IAPI-002 (Player::update). The actual Macroquad key→InputState mapping is
// performed by the Playing state before calling Player::update.

/// Per-frame keyboard input snapshot consumed by Player::update and other systems.
///
/// Fields are `pub` to allow direct construction in tests and by the Playing state.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    /// Left directional input (ArrowLeft or A).
    pub left: bool,
    /// Right directional input (ArrowRight or D).
    pub right: bool,
    /// Jump button currently held (Space).
    pub jump: bool,
    /// Jump button just pressed this frame (Space edge-triggered).
    pub jump_just: bool,
    /// Sprint modifier held (Shift).
    pub sprint: bool,
    /// Escape key just pressed (edge-triggered, for menu toggle).
    pub esc_just: bool,
    /// Confirm action key (Enter, edge-triggered).
    pub confirm: bool,
}


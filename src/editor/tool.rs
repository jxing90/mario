// Editor tool palette and drag target types.

// ============================================================================
// Tool palette
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tool {
    Drag,
    View,
    Platform,
    Spike,
    Coin,
    QuestionBlock,
    Brick,
    Enemy,
    DartEnemy,
    OscFireball,
    Checkpoint,
    Flagpole,
    PlayerSpawn,
    Eraser,
}

impl Tool {
    pub(crate) const ALL: &[Tool] = &[
        Tool::Drag,
        Tool::View,
        Tool::Platform,
        Tool::Spike,
        Tool::Coin,
        Tool::QuestionBlock,
        Tool::Brick,
        Tool::Enemy,
        Tool::DartEnemy,
        Tool::OscFireball,
        Tool::Checkpoint,
        Tool::Flagpole,
        Tool::PlayerSpawn,
        Tool::Eraser,
    ];

    pub(crate) fn name(&self) -> &str {
        match self {
            Tool::Drag => "Drag",
            Tool::View => "View",
            Tool::Platform => "Platform",
            Tool::Spike => "Spike",
            Tool::Coin => "Coin",
            Tool::QuestionBlock => "QBlock",
            Tool::Brick => "Brick",
            Tool::Enemy => "Enemy",
            Tool::DartEnemy => "DartEnemy",
            Tool::OscFireball => "OFireball",
            Tool::Checkpoint => "ChkPt",
            Tool::Flagpole => "Flagpole",
            Tool::PlayerSpawn => "Player",
            Tool::Eraser => "Eraser",
        }
    }

    pub(crate) fn shortcut(&self) -> &str {
        match self {
            Tool::Drag => "D",
            Tool::View => "V",
            Tool::Platform => "1",
            Tool::Spike => "2",
            Tool::Coin => "3",
            Tool::QuestionBlock => "4",
            Tool::Brick => "5",
            Tool::Enemy => "6",
            Tool::DartEnemy => "7",
            Tool::OscFireball => "8",
            Tool::Checkpoint => "9",
            Tool::Flagpole => "0",
            Tool::PlayerSpawn => "P",
            Tool::Eraser => "Del",
        }
    }
}

/// What kind of entity is being dragged.
#[derive(Debug, Clone, Copy)]
pub(crate) enum DragTarget {
    Platform(usize),
    Spike(usize),
    Coin(usize),
    QuestionBlock(usize),
    Brick(usize),
    Enemy(usize),
    DartEnemy(usize),
    OscFireball(usize),
    Checkpoint(usize),
    PlayerSpawn,
    Flagpole,
}

// ============================================================================
// Shared constants
// ============================================================================

pub(crate) const MENU_BAR: &[(&str, &[(&str, &str)])] = &[
    ("File", &[
        ("New          Ctrl+N", "new"),
        ("Open", "open_header"),
        ("Open File...", "open_file"),
        ("Save         Ctrl+S", "save"),
        ("Save As...   Ctrl+Shift+S", "save_as"),
        ("Rename...    Ctrl+R", "rename"),
    ]),
    ("View", &[
        ("Zoom In      +", "zoom_in"),
        ("Zoom Out     -", "zoom_out"),
        ("Reset Zoom", "zoom_reset"),
        ("Grid Snap    G", "grid"),
    ]),
];

/// Shared value types used across PhraseDJ crates.
///
/// Keeping these in `pdj-core` ensures there is one definition of each type
/// that all crates agree on without circular dependencies.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Identifiers
// ---------------------------------------------------------------------------

/// Unique identifier for a track in the library.
///
/// Generated once on import and stored in the database.  Stable across moves
/// and renames as long as the file content hash matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackId(Uuid);

impl TrackId {
    /// Create a new random TrackId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse from a UUID string (e.g. from the database).
    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for TrackId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TrackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// Deck index
// ---------------------------------------------------------------------------

/// Which of the two playback decks an operation targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Deck {
    A,
    B,
}

impl Deck {
    /// Numeric index — useful for array indexing in the engine.
    pub fn index(self) -> usize {
        match self {
            Deck::A => 0,
            Deck::B => 1,
        }
    }
}

// ---------------------------------------------------------------------------
// Audio parameters
// ---------------------------------------------------------------------------

/// A normalised gain / fader value in the range [0.0, 1.0].
///
/// Values outside this range are clamped.  Use `Gain::new` for safe construction.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Gain(f32);

impl Gain {
    /// Construct a Gain, clamping to [0, 1].
    pub fn new(v: f32) -> Self {
        Self(v.clamp(0.0, 1.0))
    }

    /// The underlying f32.
    pub fn value(self) -> f32 {
        self.0
    }
}

impl Default for Gain {
    fn default() -> Self {
        Self(1.0)
    }
}

// ---------------------------------------------------------------------------
// Song structure
// ---------------------------------------------------------------------------

/// A phrase kind produced by the AI phrase-detection model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhraseKind {
    Intro,
    Verse,
    Chorus,
    Drop,
    Outro,
    Break,
}

impl std::fmt::Display for PhraseKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PhraseKind::Intro => "intro",
            PhraseKind::Verse => "verse",
            PhraseKind::Chorus => "chorus",
            PhraseKind::Drop => "drop",
            PhraseKind::Outro => "outro",
            PhraseKind::Break => "break",
        };
        write!(f, "{s}")
    }
}

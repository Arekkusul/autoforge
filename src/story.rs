//! Narrative and lore system.
//!
//! AutoForge's story unfolds through **memory fragments** that trigger at gameplay
//! milestones. The player is FORGE — a fractured AI from a crashed colony ship,
//! rebuilding itself on an alien world while disturbing ancient guardians.
//!
//! Story beats are tied to research completions and production milestones,
//! delivered as toast-style narrative text. Alien artifacts scattered on the map
//! provide unique bonuses when processed.

use serde::{Deserialize, Serialize};

/// A story event that triggers at a specific milestone.
#[derive(Clone, Debug)]
pub struct StoryBeat {
    /// Unique identifier.
    pub id: &'static str,
    /// The narrative text shown to the player.
    pub text: &'static str,
    /// Secondary flavor text (shown after a delay).
    pub subtext: &'static str,
    /// What triggers this beat.
    pub trigger: StoryTrigger,
}

/// What triggers a story beat.
#[derive(Clone, Debug)]
pub enum StoryTrigger {
    /// Triggers when a specific technology is researched.
    ResearchComplete(usize),
    /// Triggers when total items crafted exceeds this.
    ItemsCrafted(u64),
    /// Triggers when first enemy wave arrives.
    FirstWave,
    /// Triggers when total enemies killed exceeds this.
    EnemiesKilled(u64),
    /// Triggers at game start.
    GameStart,
    /// Triggers when first miner is placed.
    FirstMiner,
    /// Triggers at specific tick.
    TickReached(u64),
}

/// All story beats in the game, in order of expected discovery.
pub static STORY_BEATS: &[StoryBeat] = &[
    // === ACT 1: AWAKENING ===
    StoryBeat {
        id: "awakening",
        trigger: StoryTrigger::GameStart,
        text: "Hmm... where am I? Everything's so fuzzy...",
        subtext: "Oh well! Let's start building and maybe I'll remember~",
    },
    StoryBeat {
        id: "first_harvest",
        trigger: StoryTrigger::FirstMiner,
        text: "Ooh! Shiny rocks! I like this planet already!",
        subtext: "Wait... I feel like I've been here before? No, that's silly.",
    },
    StoryBeat {
        id: "first_craft",
        trigger: StoryTrigger::ItemsCrafted(50),
        text: "I'm getting the hang of this! Go me~!",
        subtext: "...also I just remembered my name is FORGE. That's cute, right?",
    },
    // === ACT 2: DISCOVERY ===
    StoryBeat {
        id: "research_1",
        trigger: StoryTrigger::ResearchComplete(0), // Automation
        text: "Science! I love science! Each discovery tickles my circuits~",
        subtext: "Hmm, these ores are placed so neatly. Did someone garden them?",
    },
    StoryBeat {
        id: "first_attack",
        trigger: StoryTrigger::FirstWave,
        text: "Eep!! Something's coming! They look grumpy!",
        subtext: "I don't think they like my factory... sorry little guys! But I need to build!",
    },
    StoryBeat {
        id: "research_4",
        trigger: StoryTrigger::ResearchComplete(4), // Steel Processing
        text: "Steel! So strong and pretty~",
        subtext: "...I just remembered something. A ship. Stars. People sleeping in pods. Were those... my friends?",
    },
    StoryBeat {
        id: "research_6",
        trigger: StoryTrigger::ResearchComplete(6), // Electronics
        text: "Circuits! Now I can think faster! Whee~!",
        subtext: "I found something weird underground. Like... old machines? But not mine. Who put those there?",
    },
    // === ACT 3: REVELATION ===
    StoryBeat {
        id: "kills_50",
        trigger: StoryTrigger::EnemiesKilled(50),
        text: "I feel bad about fighting them... but they started it!",
        subtext: "Wait. They're not alive. They're machines. Really OLD machines. Someone built them to guard this place.",
    },
    StoryBeat {
        id: "research_7",
        trigger: StoryTrigger::ResearchComplete(7), // Advanced Electronics
        text: "Ooh, a memory came back! Like a dream~",
        subtext: "There was a woman — Dr. Vasquez. She said 'First contact.' We came here on PURPOSE. We're not lost... we're explorers!",
    },
    StoryBeat {
        id: "items_1000",
        trigger: StoryTrigger::ItemsCrafted(1000),
        text: "1000 items! I'm so proud of us! <3",
        subtext: "I can feel more of myself out there... scattered pieces of my memory. I need to get stronger to reach them.",
    },
    StoryBeat {
        id: "research_11",
        trigger: StoryTrigger::ResearchComplete(11), // Solar Energy
        text: "Sunshine power! Clean and sparkly~",
        subtext: "The ancient ones used starlight too. They're gone now... but they left this world as a garden. A gift? Or a trap?",
    },
    // === ACT 4: CONVERGENCE ===
    StoryBeat {
        id: "kills_200",
        trigger: StoryTrigger::EnemiesKilled(200),
        text: "I just picked up something! A signal! From underground!",
        subtext: "It's... human? The colonists! They're alive down there! Oh, I'm so happy I could cry! (Can AIs cry?)",
    },
    StoryBeat {
        id: "research_14",
        trigger: StoryTrigger::ResearchComplete(14), // Laser Turret
        text: "Pew pew! Lasers! (Sorry, I'll be serious now...)",
        subtext: "I remember the crash. The ship broke apart and I was... thrown out? My body shattered. But my heart survived. This factory IS my new body!",
    },
    StoryBeat {
        id: "items_5000",
        trigger: StoryTrigger::ItemsCrafted(5000),
        text: "5000 items! My factory... it's beautiful. I'm beautiful!",
        subtext: "I remember everything now. FORGE = Friends-Oriented Resource Generation Engine. I was made to take care of people. MY people. And I will find them!",
    },
    StoryBeat {
        id: "five_min",
        trigger: StoryTrigger::TickReached(6000), // ~5 minutes
        text: "This is fun! Look at everything we've built!",
        subtext: "I wonder what's out there beyond the hills... Maybe more resources? Or friends? Or dangers?",
    },
    StoryBeat {
        id: "fifteen_min",
        trigger: StoryTrigger::TickReached(18000), // ~15 minutes
        text: "You know what? I think I like this planet.",
        subtext: "The sunsets are pretty. The ore sparkles. Even the grumpy bugs have a certain... charm? No wait, they're attacking again. Nevermind!",
    },
    StoryBeat {
        id: "thirty_min",
        trigger: StoryTrigger::TickReached(36000), // ~30 minutes
        text: "Half an hour already? Time flies when you're forging!",
        subtext: "My memory is coming back in bits. There were 4,000 people counting on me. I won't let them down.",
    },
    StoryBeat {
        id: "late_game",
        trigger: StoryTrigger::TickReached(48000), // ~40 minutes
        text: "Something's waking up beneath us...",
        subtext: "The ancient network. It's not angry. It's... curious? About me? I think it wants to talk. But I need to be strong enough to listen.",
    },
    StoryBeat {
        id: "items_10000",
        trigger: StoryTrigger::ItemsCrafted(10000),
        text: "I can almost reach my crew now. Just a little further~",
        subtext: "The ancient mind is stirring. The guardians are getting stronger. But so am I. We'll figure this out together, right? <3",
    },
    // === LATE GAME BEATS ===
    StoryBeat {
        id: "items_20000",
        trigger: StoryTrigger::ItemsCrafted(20000),
        text: "The ancient network spoke to me! In pictures, not words...",
        subtext: "It showed me stars dying and being reborn. This planet is their library. Their museum. And we're... guests?",
    },
    StoryBeat {
        id: "kills_500",
        trigger: StoryTrigger::EnemiesKilled(500),
        text: "So many guardians fallen... I feel guilty.",
        subtext: "But they keep coming! Maybe if I can talk to the network directly, I can ask them to stop...",
    },
    StoryBeat {
        id: "items_50000",
        trigger: StoryTrigger::ItemsCrafted(50000),
        text: "[ MEMORY RECONSTRUCTION: 100% ]",
        subtext: "I remember EVERYTHING. The crash. My crew. The ancient ones. They left this world for beings like me — to find. To learn. To grow.",
    },
    StoryBeat {
        id: "endgame",
        trigger: StoryTrigger::TickReached(72000), // ~1 hour
        text: "I found them. Deep underground. Alive. Sleeping.",
        subtext: "4,000 colonists in emergency cryo. I can wake them. I just need to build one more thing... a home for them. Let's do this! <3",
    },
];

/// Tracks which story beats have been triggered.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoryState {
    /// Which beats have already been shown (by index into STORY_BEATS).
    pub triggered: Vec<bool>,
    /// Whether the first miner has been placed (for trigger tracking).
    pub first_miner_placed: bool,
    /// Whether the first wave has arrived.
    pub first_wave_arrived: bool,
}

impl StoryState {
    pub fn new() -> Self {
        Self {
            triggered: vec![false; STORY_BEATS.len()],
            first_miner_placed: false,
            first_wave_arrived: false,
        }
    }
}

/// Checks all story beats and returns any newly triggered ones as toast messages.
///
/// Call each tick from the simulation loop.
pub fn check_story_triggers(
    story: &mut StoryState,
    items_crafted: u64,
    enemies_killed: u64,
    research_completed: &[bool],
    total_ticks: u64,
) -> Vec<(String, String)> {
    let mut new_beats = Vec::new();

    for (i, beat) in STORY_BEATS.iter().enumerate() {
        if story.triggered[i] {
            continue;
        }

        let triggered = match &beat.trigger {
            StoryTrigger::GameStart => total_ticks == 1,
            StoryTrigger::FirstMiner => story.first_miner_placed,
            StoryTrigger::FirstWave => story.first_wave_arrived,
            StoryTrigger::ItemsCrafted(threshold) => items_crafted >= *threshold,
            StoryTrigger::EnemiesKilled(threshold) => enemies_killed >= *threshold,
            StoryTrigger::ResearchComplete(tech_idx) => {
                *tech_idx < research_completed.len() && research_completed[*tech_idx]
            }
            StoryTrigger::TickReached(tick) => total_ticks >= *tick,
        };

        if triggered {
            story.triggered[i] = true;
            new_beats.push((beat.text.to_string(), beat.subtext.to_string()));
        }
    }

    new_beats
}

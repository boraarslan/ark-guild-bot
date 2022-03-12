pub mod commands;
pub mod database;

use std::fmt::{Debug, Display};
use entity::*;


#[derive(Debug, poise::ChoiceParameter, Clone, Copy)]
pub enum Class {
    Berserker,
    Paladin,
    Gunlancer,
    Striker,
    Wardancer,
    Scrapper,
    Soulfist,
    Gunslinger,
    Artillerist,
    Deadeye,
    Sharpshooter,
    Bard,
    Sorceress,
    Shadowhunter,
    Deathblade,
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
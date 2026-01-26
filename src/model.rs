use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Origin {
    #[default]
    Human,
    Elf,
    HalfElf,
    Dwarf,
    Orc,
    Gnome,
    Halfling,
    GiantKin,
}

impl Origin {
    pub fn all() -> [Origin; 8] {
        [
            Origin::Human,
            Origin::Elf,
            Origin::HalfElf,
            Origin::Dwarf,
            Origin::Orc,
            Origin::Gnome,
            Origin::Halfling,
            Origin::GiantKin,
        ]
    }
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Origin::Human => "Human",
                Origin::Elf => "Elf",
                Origin::HalfElf => "Half-Elf",
                Origin::Dwarf => "Dwarf",
                Origin::Orc => "Orc",
                Origin::Gnome => "Gnome",
                Origin::Halfling => "Halfling",
                Origin::GiantKin => "Giant-kin",
            }
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attributes {
    pub strength: i32,
    pub dexterity: i32,
    pub endurance: i32,
    pub faith: i32,
    pub will: i32,
    pub intelligence: i32,
    pub luck: i32,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            strength: 1,
            dexterity: 1,
            endurance: 1,
            faith: 1,
            will: 1,
            intelligence: 1,
            luck: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum AbilityType {
    #[default]
    Passive,
    Maneuver,
    Spell,
    Miracle,
}

impl std::fmt::Display for AbilityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl AbilityType {
    pub fn all() -> [AbilityType; 4] {
        [
            AbilityType::Passive,
            AbilityType::Maneuver,
            AbilityType::Spell,
            AbilityType::Miracle,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ability {
    pub name: String,
    pub description: String,
    pub ability_type: AbilityType,
    pub tags: String, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub name: String,
    pub level: i32,
    pub origin: Origin,
    pub attributes: Attributes,
    pub current_hp: i32,
    pub wounds: i32,
    pub xp: i32,
    pub tender: i32,
    pub armor_bonus: i32,
    pub expended_spell_slots: i32,
    pub expended_miracle_slots: i32,
    pub inventory: Vec<String>,
    pub abilities: Vec<Ability>,
    pub notes: String,
}

impl Default for Character {
    fn default() -> Self {
        let attrs = Attributes::default();
        let origin = Origin::default();
        Self {
            name: "New Character".to_string(),
            level: 1,
            origin,
            attributes: attrs,
            current_hp: 5, 
            wounds: 0,
            xp: 0,
            tender: 200,
            armor_bonus: 0,
            expended_spell_slots: 0,
            expended_miracle_slots: 0,
            inventory: vec![String::new(); 5],
            abilities: Vec::new(),
            notes: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_serialization() {
        let char = Character::default();
        let serialized = serde_json::to_string(&char).expect("Should serialize");
        let deserialized: Character = serde_json::from_str(&serialized).expect("Should deserialize");
        
        assert_eq!(char.name, deserialized.name);
        assert_eq!(char.attributes.strength, deserialized.attributes.strength);
        assert_eq!(char.origin, deserialized.origin);
    }
}

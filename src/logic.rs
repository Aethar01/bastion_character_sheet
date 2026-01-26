use crate::model::{Character, Origin};

pub fn calculate_max_hp(char: &Character) -> i32 {
    let base = 2 * char.level + 3 * char.attributes.endurance;
    let max = match char.origin {
        Origin::GiantKin => 3 * char.level + 3 * char.attributes.endurance,
        _ => base,
    };
    (max - (char.wounds * char.attributes.endurance)).max(1)
}

pub fn calculate_movement_speed(char: &Character) -> i32 {
    let dex_half = (char.attributes.dexterity as f32 / 2.0).ceil() as i32;
    2 + dex_half
}

pub fn calculate_carrying_slots(char: &Character) -> i32 {
    let base = 4 + char.attributes.strength;
    match char.origin {
        Origin::Dwarf => 6 + char.attributes.strength,
        _ => base,
    }
}

pub fn calculate_prepared_slots(char: &Character) -> i32 {
    2 + char.level
}

pub fn calculate_spell_slots(char: &Character) -> i32 {
    char.attributes.intelligence
}

pub fn calculate_miracle_slots(char: &Character) -> i32 {
    char.attributes.faith
}

pub fn calculate_armor_class(char: &Character) -> i32 {
    6 + char.attributes.dexterity + char.armor_bonus
}

pub fn calculate_crit_range(char: &Character) -> i32 {
    let bonus = char.attributes.luck / 2;
    12 - bonus
}

pub fn get_origin_traits(origin: Origin) -> Vec<&'static str> {
    match origin {
        Origin::Human => vec!["Humans starting Luck is 3."],
        Origin::Elf => vec![
            "Elves can sense magic near to them, they can see the aura given off by magic creatures, items, spells, and miracles within 2 spaces.",
            "Focus: Elves can focus as an action, expanding the range of their magical sight to 10 spaces for a Short duration.",
        ],
        Origin::HalfElf => vec![
            "Half-Elves, due to their elvish lineage have enhanced eyesight. Transparent obstructions do not obscure their vision.",
            "Prodigy: When you roll a Skill Check, Attack or Saving Throw, you can cast a spell to grant +1d6 Circumstance.",
        ],
        Origin::Dwarf => vec![
            "Dwarves don’t need to meet the Strength requirement for armour, and slow armour does not impede their movement.",
            "Dwarves have 6 + Strength Carrying Slots.",
        ],
        Origin::Orc => vec![
            "Orcs can see in the dark as well as they can in light, though only in black and white.",
            "Charge: When using a move action, you can move double your speed if it is in a straight line towards an Enemy.",
        ],
        Origin::Gnome => vec![
            "Gnomes are small. As a consequence, treat their Strength as 2 lower for the purposes of overcoming the Heavy property of weapons.",
            "The magic of the forest suffuses Gnomes, allowing them to roll twice to resist the effects of spells (but not miracles), taking the best result, and also providing them the ability to communicate with small animals.",
        ],
        Origin::Halfling => vec![
            "Halflings are small. As a consequence, treat their Strength as 2 lower for the purposes of overcoming the Heavy property of weapons.", "Move through spaces of >Small creatures",
            "Halflings can move through spaces occupied by creatures bigger than small, and they can fit into smaller spaces than other origins.",
        ],
        Origin::GiantKin => vec![
            "Giant-kin toughness benefit them with maximum Hit Points of 3 × Level + 3 × Endurance.",
            "Giant-kin are large. As a consequence, treat their Strength as 2 higher for the purposes of overcoming the Heavy property of weapons. Additionally, armour for Giant-kin costs 10% more.",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Character, Origin};

    #[test]
    fn test_hp_calculation() {
        let mut char = Character::default();
        char.level = 1;
        char.attributes.endurance = 2;
        
        assert_eq!(calculate_max_hp(&char), 8);

        char.origin = Origin::GiantKin;
        assert_eq!(calculate_max_hp(&char), 9);
    }

    #[test]
    fn test_hp_calculation_with_wounds() {
        let mut char = Character::default();
        char.level = 1;
        char.attributes.endurance = 2;
        // Base: 8
        
        char.wounds = 1;
        // 8 - (1 * 2) = 6
        assert_eq!(calculate_max_hp(&char), 6);

        char.wounds = 3;
        // 8 - (3 * 2) = 2
        assert_eq!(calculate_max_hp(&char), 2);

        char.wounds = 4;
        // 8 - (4 * 2) = 0 -> clamped to 1
        assert_eq!(calculate_max_hp(&char), 1);
    }

    #[test]
    fn test_carrying_capacity() {
        let mut char = Character::default();
        char.attributes.strength = 3;

        assert_eq!(calculate_carrying_slots(&char), 7);

        char.origin = Origin::Dwarf;
        assert_eq!(calculate_carrying_slots(&char), 9);
    }

    #[test]
    fn test_movement_speed() {
        let mut char = Character::default();
        
        char.attributes.dexterity = 1;
        assert_eq!(calculate_movement_speed(&char), 3);

        char.attributes.dexterity = 2;
        assert_eq!(calculate_movement_speed(&char), 3);

        char.attributes.dexterity = 3;
        assert_eq!(calculate_movement_speed(&char), 4);
    }

    #[test]
    fn test_crit_range() {
        let mut char = Character::default();
        
        char.attributes.luck = 0;
        assert_eq!(calculate_crit_range(&char), 12);

        char.attributes.luck = 1;
        assert_eq!(calculate_crit_range(&char), 12);

        char.attributes.luck = 2;
        assert_eq!(calculate_crit_range(&char), 11);

        char.attributes.luck = 6;
        assert_eq!(calculate_crit_range(&char), 9);
    }
}
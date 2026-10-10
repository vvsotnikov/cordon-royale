use serde::{Deserialize, Serialize};
#[derive(Serialize)]
pub struct Item {
    pub name: &'static str,
    pub slot: &'static str,
    pub power: u64,
    pub price: u64,
}
pub const ITEMS: [Item; 8] = [
    Item {
        name: "PM pistol",
        slot: "weapon",
        power: 0,
        price: 0,
    },
    Item {
        name: "TOZ shotgun",
        slot: "weapon",
        power: 30,
        price: 15,
    },
    Item {
        name: "AK-74",
        slot: "weapon",
        power: 150,
        price: 100,
    },
    Item {
        name: "LR-300",
        slot: "weapon",
        power: 700,
        price: 500,
    },
    Item {
        name: "Gauss rifle",
        slot: "weapon",
        power: 3000,
        price: 2500,
    },
    Item {
        name: "Leather jacket",
        slot: "armor",
        power: 0,
        price: 0,
    },
    Item {
        name: "SEVA suit",
        slot: "armor",
        power: 200,
        price: 180,
    },
    Item {
        name: "Exoskeleton",
        slot: "armor",
        power: 1500,
        price: 1200,
    },
];
pub const RANKS: [&str; 7] = [
    "Rookie", "Stalker", "Hunter", "Veteran", "Expert", "Master", "Legend",
];
pub const PROMOTIONS: [u64; 6] = [5, 15, 40, 100, 250, 600];
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Profile {
    pub name: String,
    pub power: u64,
    pub rubles: u64,
    pub wins: u64,
    pub rank: usize,
    pub inventory: Vec<usize>,
    pub weapon: usize,
    pub armor: usize,
    pub trophies: Vec<usize>,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "Stalker".into(),
            power: 0,
            rubles: 0,
            wins: 0,
            rank: 0,
            inventory: vec![0, 5],
            weapon: 0,
            armor: 5,
            trophies: vec![],
        }
    }
}
impl Profile {
    pub fn sanitize(&mut self) {
        self.rank = self.rank.min(RANKS.len() - 1);
        self.inventory.retain(|&n| n < ITEMS.len());
        self.inventory.extend([0, 5]);
        self.inventory.sort_unstable();
        self.inventory.dedup();
        if !self.inventory.contains(&self.weapon) || ITEMS[self.weapon].slot != "weapon" {
            self.weapon = 0;
        }
        if !self.inventory.contains(&self.armor) || ITEMS[self.armor].slot != "armor" {
            self.armor = 5;
        }
        self.name = self
            .name
            .chars()
            .filter(|c| c.is_alphanumeric() || " _-".contains(*c))
            .take(20)
            .collect();
        if self.name.is_empty() {
            self.name = "Stalker".into();
        }
    }
    pub fn gain(&self) -> u64 {
        1_u64 << self.rank
    }
    pub fn strength(&self) -> u64 {
        self.power
            .saturating_add(ITEMS[self.weapon].power)
            .saturating_add(ITEMS[self.armor].power)
    }
    pub fn train(&mut self) -> u64 {
        let gain = self.gain();
        self.power = self.power.saturating_add(gain).min(1_000_000_000_000);
        gain
    }
    pub fn buy(&mut self, n: usize) -> Result<(), &'static str> {
        if n >= ITEMS.len() {
            return Err("Unknown item.");
        }
        if self.inventory.contains(&n) {
            return Err("You already own this item.");
        }
        if self.rubles < ITEMS[n].price {
            return Err("Earn more rubles by defeating enemies.");
        }
        self.rubles -= ITEMS[n].price;
        self.inventory.push(n);
        self.equip(n)
    }
    pub fn equip(&mut self, n: usize) -> Result<(), &'static str> {
        if n >= ITEMS.len() || !self.inventory.contains(&n) {
            return Err("You do not own this item.");
        }
        if ITEMS[n].slot == "weapon" {
            self.weapon = n
        } else {
            self.armor = n
        }
        Ok(())
    }
    pub fn promote(&mut self) -> Result<(), &'static str> {
        if self.rank >= PROMOTIONS.len() {
            return Err("You are already a Legend.");
        }
        if self.wins < PROMOTIONS[self.rank] {
            return Err("Earn more wins before promotion.");
        }
        self.wins -= PROMOTIONS[self.rank];
        self.rank += 1;
        self.power = 0;
        Ok(())
    }
    pub fn reward(
        &mut self,
        enemy: usize,
        wins: u64,
        rubles: u64,
        drop: Option<usize>,
    ) -> Option<&'static str> {
        self.wins = self.wins.saturating_add(wins);
        self.rubles = self.rubles.saturating_add(rubles);
        if self.trophies.contains(&enemy) {
            return None;
        }
        self.trophies.push(enemy);
        if let Some(n) = drop.filter(|n| !self.inventory.contains(n)) {
            self.inventory.push(n);
            Some(ITEMS[n].name)
        } else {
            None
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inventory_requires_ownership_and_spends_exact_price() {
        let mut p = Profile::default();
        assert!(p.equip(4).is_err());
        assert!(p.buy(1).is_err());
        p.rubles = 15;
        p.buy(1).unwrap();
        assert_eq!(p.rubles, 0);
        assert_eq!(p.strength(), 30);
        assert!(p.buy(1).is_err());
        p.equip(0).unwrap();
        assert_eq!(p.strength(), 0);
    }
    #[test]
    fn promotion_keeps_gear_and_resets_fieldcraft() {
        let mut p = Profile {
            power: 100,
            wins: 5,
            rubles: 15,
            ..Default::default()
        };
        p.buy(1).unwrap();
        p.promote().unwrap();
        assert_eq!(p.power, 0);
        assert_eq!(p.strength(), 30);
        assert_eq!(p.train(), 2);
        assert!(p.promote().is_err());
    }
    #[test]
    fn first_kill_loot_is_owned_once() {
        let mut p = Profile::default();
        assert_eq!(p.reward(1, 3, 50, Some(1)), Some("TOZ shotgun"));
        assert_eq!(p.reward(1, 3, 50, Some(1)), None);
        assert_eq!(p.wins, 6);
        assert_eq!(p.inventory.iter().filter(|&&n| n == 1).count(), 1);
        let saved = serde_json::to_string(&p).unwrap();
        let restored: Profile = serde_json::from_str(&saved).unwrap();
        assert!(restored.inventory.contains(&1));
    }
}

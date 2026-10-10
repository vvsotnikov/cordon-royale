use serde::{Deserialize, Serialize};
#[derive(Serialize)]
pub struct Item {
    pub name: &'static str,
    pub slot: &'static str,
    pub power: u64,
}
pub const ITEMS: [Item; 8] = [
    Item {
        name: "PM pistol",
        slot: "weapon",
        power: 0,
    },
    Item {
        name: "TOZ shotgun",
        slot: "weapon",
        power: 30,
    },
    Item {
        name: "AK-74",
        slot: "weapon",
        power: 150,
    },
    Item {
        name: "LR-300",
        slot: "weapon",
        power: 700,
    },
    Item {
        name: "Gauss rifle",
        slot: "weapon",
        power: 3000,
    },
    Item {
        name: "Leather jacket",
        slot: "armor",
        power: 0,
    },
    Item {
        name: "SEVA suit",
        slot: "armor",
        power: 200,
    },
    Item {
        name: "Exoskeleton",
        slot: "armor",
        power: 1500,
    },
];

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct Profile {
    pub name: String,
    pub power: u64,
    pub exp: u64,
    pub lootboxes: u64,
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
            exp: 0,
            lootboxes: 0,
            inventory: vec![0, 5],
            weapon: 0,
            armor: 5,
            trophies: vec![],
        }
    }
}
impl Profile {
    pub fn sanitize(&mut self) {
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
        self.trophies.retain(|&n| n < 5);
        self.trophies.sort_unstable();
        self.trophies.dedup();
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
        1
    }
    pub fn strength(&self) -> u64 {
        self.power
            .saturating_add(ITEMS[self.weapon].power)
            .saturating_add(ITEMS[self.armor].power)
    }
    pub fn train(&mut self) {
        self.power = self
            .power
            .saturating_add(self.gain())
            .min(1_000_000_000_000);
    }
    pub fn unlocked(&self, stage: usize) -> bool {
        stage == 0 || self.trophies.contains(&(stage - 1))
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
    pub fn victory(&mut self, stage: usize, roll: u32) -> bool {
        if !self.trophies.contains(&stage) {
            self.trophies.push(stage);
        }
        let drop = roll < 5;
        if drop {
            self.lootboxes = self.lootboxes.saturating_add(1);
        }
        drop
    }
    pub fn open_box(&mut self, roll: u32) -> Result<&'static str, &'static str> {
        if self.lootboxes == 0 {
            return Err("No lootboxes. Each victory has a 5% drop chance.");
        }
        self.lootboxes -= 1;
        let pool = [1, 2, 3, 4, 6, 7];
        let missing: Vec<_> = pool
            .into_iter()
            .filter(|n| !self.inventory.contains(n))
            .collect();
        if missing.is_empty() {
            return Ok("Collection complete — duplicate equipment.");
        }
        let n = missing[roll as usize % missing.len()];
        self.inventory.push(n);
        Ok(ITEMS[n].name)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unlocks_and_rare_loot() {
        let mut p = Profile::default();
        assert!(p.unlocked(0));
        assert!(!p.unlocked(1));
        assert!(!p.victory(0, 5));
        assert!(p.unlocked(1));
        assert_eq!(p.lootboxes, 0);
        assert!(p.victory(0, 4));
        assert_eq!(p.lootboxes, 1);
        assert!(p.equip(4).is_err());
        p.open_box(0).unwrap();
        assert!(p.inventory.contains(&1));
        assert_eq!(p.lootboxes, 0);
        assert!(p.open_box(0).is_err());
        p.equip(1).unwrap();
        assert_eq!(p.strength(), 30);
    }
    #[test]
    fn old_profile_migrates() {
        let mut p:Profile=serde_json::from_str(r#"{"name":"Old","power":100,"wins":3,"rubles":90,"rank":2,"inventory":[0,1,5],"trophies":[0]}"#).unwrap();
        p.sanitize();
        assert_eq!(p.exp, 0);
        assert_eq!(p.power, 100);
        assert!(p.inventory.contains(&1));
        assert!(p.unlocked(1));
    }
}

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

#[derive(Serialize)]
pub struct ArmorTier {
    pub name: &'static str,
    pub cost: u64,
    pub gain: u64,
    pub model: &'static str,
}
pub const ARMORS: [ArmorTier; 6] = [
    ArmorTier {
        name: "No armor",
        cost: 0,
        gain: 1,
        model: "player_0",
    },
    ArmorTier {
        name: "Leather jacket",
        cost: 5,
        gain: 3,
        model: "player_1",
    },
    ArmorTier {
        name: "Sunrise suit",
        cost: 25,
        gain: 10,
        model: "player_2",
    },
    ArmorTier {
        name: "SEVA suit",
        cost: 100,
        gain: 35,
        model: "player_3",
    },
    ArmorTier {
        name: "Exoskeleton",
        cost: 500,
        gain: 100,
        model: "player_4",
    },
    ArmorTier {
        name: "Nosorog exoskeleton",
        cost: 2000,
        gain: 300,
        model: "player_5",
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
    pub run_wins: Vec<usize>,
    pub ascension: usize,
    pub rebirths: u32,
    pub training_tenths: u64,
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
            run_wins: vec![],
            ascension: 0,
            rebirths: 0,
            training_tenths: 0,
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
        self.rebirths = self.rebirths.min(100_000);
        self.training_tenths %= 10;
        self.ascension = self.ascension.min(ARMORS.len() - 1);
        self.run_wins.retain(|&n| n < 5);
        self.run_wins.sort_unstable();
        self.run_wins.dedup();
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
    pub fn gain(&self) -> f64 {
        self.gain_tenths() as f64 / 10.
    }
    fn gain_tenths(&self) -> u64 {
        ARMORS[self.ascension].gain * (10 + self.rebirths as u64)
    }
    pub fn rebirth_requirement(&self) -> u64 {
        100 * (self.rebirths as u64 + 1).pow(2)
    }
    pub fn rebirth(&mut self) -> Result<(), &'static str> {
        if self.rebirths >= 100_000 {
            return Err("Maximum rebirths reached.");
        }
        if self.power < self.rebirth_requirement() {
            return Err("Train more power before rebirthing. Equipment does not count.");
        }
        self.rebirths += 1;
        self.power = 0;
        self.training_tenths = 0;
        self.run_wins.clear();
        Ok(())
    }
    pub fn ascend(&mut self) -> Result<(), &'static str> {
        let next = ARMORS
            .get(self.ascension + 1)
            .ok_or("Already wearing Nosorog armor.")?;
        if self.exp < next.cost {
            return Err("Earn more EXP to ascend.");
        }
        self.exp -= next.cost;
        self.ascension += 1;
        Ok(())
    }
    pub fn reward_unlocked(&self, stage: usize) -> bool {
        self.unlocked(stage) && !self.run_wins.is_empty()
    }
    pub fn strength(&self) -> u64 {
        self.power
            .saturating_add(ITEMS[self.weapon].power)
            .saturating_add(ITEMS[self.armor].power)
    }
    pub fn train(&mut self) {
        let tenths = self.training_tenths + self.gain_tenths();
        self.power = self
            .power
            .saturating_add(tenths / 10)
            .min(1_000_000_000_000);
        self.training_tenths = tenths % 10;
    }
    pub fn unlocked(&self, stage: usize) -> bool {
        stage == 0 || self.run_wins.contains(&(stage - 1))
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
        let first = !self.run_wins.contains(&stage);
        if first {
            self.run_wins.push(stage);
        }
        let drop = first && roll < 5;
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
        let pool = [1, 2, 3, 4];
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
    fn rebirth_preserves_rewards_and_fractional_training() {
        let mut p = Profile::default();
        p.inventory.extend([4, 7]);
        p.weapon = 4;
        p.armor = 7;
        p.power = 99;
        assert!(p.rebirth().is_err(), "equipment cannot satisfy threshold");
        p.power = 100;
        p.exp = 7;
        p.ascension = 1;
        p.lootboxes = 2;
        p.victory(0, 99);
        p.rebirth().unwrap();
        assert_eq!(p.power, 0);
        assert!(p.run_wins.is_empty());
        assert_eq!(
            (p.exp, p.ascension, p.lootboxes, p.weapon, p.armor),
            (7, 1, 2, 4, 7)
        );
        assert_eq!(p.trophies, vec![0]);
        assert_eq!(p.inventory, vec![0, 5, 4, 7]);
        assert_eq!(p.gain(), 3.3);
        assert_eq!(p.rebirth_requirement(), 400);
        for _ in 0..3 {
            p.train();
        }
        let mut p: Profile = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        p.sanitize();
        for _ in 0..7 {
            p.train();
        }
        assert_eq!(p.power, 33);
        assert_eq!(p.training_tenths, 0);
        assert!(p.rebirth().is_err());
        p.power = 400;
        p.rebirth().unwrap();
        assert_eq!(p.gain(), 3.6);
        assert_eq!(p.rebirth_requirement(), 900);
    }
    #[test]
    fn unlocks_and_rare_loot() {
        let mut p = Profile::default();
        assert!(p.unlocked(0));
        assert!(!p.unlocked(1));
        assert!(!p.victory(0, 5));
        assert!(p.unlocked(1));
        assert_eq!(p.lootboxes, 0);
        assert!(!p.victory(0, 4), "one loot roll per stage per run");
        p.run_wins.clear();
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
    fn ascension_costs_and_persistence() {
        let mut p = Profile::default();
        p.power = 123;
        p.exp = 2630;
        p.victory(0, 99);
        for tier in 1..ARMORS.len() {
            let exp = p.exp;
            p.ascend().unwrap();
            assert_eq!(p.exp, exp - ARMORS[tier].cost);
            assert_eq!(p.gain(), ARMORS[tier].gain as f64);
            assert_eq!(p.power, 123);
            assert!(p.unlocked(1));
        }
        assert_eq!(p.exp, 0);
        assert!(p.ascend().is_err());
        let mut restored: Profile =
            serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
        restored.sanitize();
        assert_eq!(restored.gain(), 300.);
        assert_eq!(restored.run_wins, p.run_wins);
        let mut poor = Profile::default();
        assert!(poor.ascend().is_err());
        assert_eq!(poor.exp, 0);
        assert_eq!(poor.ascension, 0);
    }
    #[test]
    fn old_profile_migrates() {
        let mut p:Profile=serde_json::from_str(r#"{"name":"Old","power":100,"wins":3,"rubles":90,"rank":2,"inventory":[0,1,5],"trophies":[0]}"#).unwrap();
        p.sanitize();
        assert_eq!(p.exp, 0);
        assert_eq!(p.power, 100);
        assert!(p.inventory.contains(&1));
        assert!(p.trophies.contains(&0));
        assert!(!p.unlocked(1), "old saves begin a fresh run");
    }
}

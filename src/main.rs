use crate::Rarity::*;
use comfy_table::Table;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;
use std::fs::read_to_string;
use std::ops::RangeInclusive;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
enum Rarity {
    Common,
    Uncommon,
    Rare,
    VeryRare,
}

impl Rarity {
    fn next_rarity(self) -> Option<Self> {
        match self {
            Common => Some(Uncommon),
            Uncommon => Some(Rare),
            Rare => Some(VeryRare),
            VeryRare => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize)]
enum Biome {
    MostTerrain,
    Coastal,
    Underdark,
    Desert,
    Mountain,
    Swamp,
    Forest,
    Arctic,
    Hills,
    Grasslands,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
struct Herb {
    name: String,
    rarity: Rarity,
    biomes: Vec<Biome>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct RarityConfig {
    price_lower: u16,
    price_upper: u16,
    likelihood: f32,
}

impl RarityConfig {
    fn price_range(&self) -> RangeInclusive<u16> {
        self.price_lower..=self.price_upper
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct RarityConfigs {
    common: RarityConfig,
    uncommon: RarityConfig,
    rare: RarityConfig,
    very_rare: RarityConfig,
}

impl RarityConfigs {
    fn config(&self, rarity: Rarity) -> &RarityConfig {
        match rarity {
            Common => &self.common,
            Uncommon => &self.uncommon,
            Rare => &self.rare,
            VeryRare => &self.very_rare,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct Config {
    local_biomes: HashSet<Biome>,
    rarities: RarityConfigs,
    herbs: Vec<Herb>,
}

struct HerbStock {
    herb: Herb,
    quantity: u16,
    price: u16,
}

fn generate_stock<R: Rng>(cfg: &Config, rng: &mut R) -> Vec<HerbStock> {
    let mut stock = Vec::new();
    for herb in cfg.herbs.iter() {
        let is_local = herb.biomes.iter().any(|b| cfg.local_biomes.contains(&b));
        let effective_rarity = if is_local {
            herb.rarity
        } else {
            if let Some(effective_rarity) = herb.rarity.next_rarity() {
                effective_rarity
            } else {
                continue;
            }
        };
        let rarity_config = cfg.rarities.config(effective_rarity);
        let mut quantity = 0;
        while rng.gen_range(0.0f32..1.0f32) < rarity_config.likelihood {
            quantity += 1;
        }
        if quantity == 0 {
            continue;
        }
        let price = rng.gen_range(rarity_config.price_range());
        stock.push(HerbStock {
            herb: herb.clone(),
            quantity,
            price,
        });
    }
    stock
}

fn main() -> Result<(), Box<dyn Error>> {
    let cfg_text = read_to_string("herb-market.config.toml")?;
    let cfg: Config = toml::from_str(cfg_text.as_str())?;

    let mut table = Table::new();
    table.set_header(["Herb", "Quantity", "Price (gp)"]);

    let mut rng = thread_rng();

    let mut stock = generate_stock(&cfg, &mut rng);
    stock.sort_by_key(|herb_stock| herb_stock.herb.name.clone());

    for herb_stock in stock {
        table.add_row([
            herb_stock.herb.name.as_str(),
            format!("{}", herb_stock.quantity).as_str(),
            format!("{}", herb_stock.price).as_str(),
        ]);
    }
    println!("```");
    println!("{table}");
    println!("```");

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd.exe")
        .arg("/c")
        .arg("pause")
        .status();

    Ok(())
}

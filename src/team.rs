use crate::caches::movesets::cache_moveset;
use crate::caches::pokemon::species_ref;
use std::rc::Rc;
use vdex::{Ability, Nature, Stat};
use vdex::items;
use vdex::moves;
use vdex::pokemon;

pub type Team = Vec<Rc<TeamMember>>;

#[derive(Copy, Clone, Debug)]
pub struct TeamMember {
    pub pokemon: &'static pokemon::Pokemon,
    pub gender: pokemon::Gender,
    pub ability: Ability,
    pub nature: Nature,
    pub held: Option<&'static items::Item>,
    pub friendship: u8,
    pub evs: pokemon::BaseStats,
    pub ivs: pokemon::BaseStats,
    pub moves: [Option<&'static moves::Move>; 4],
    pub pp_ups: [u8; 4],
    pub level: u8,
}

impl TeamMember {
    pub fn max_pp(&self, slot: u8) -> u8 {
        if let Some(mov) = self.moves[slot as usize] {
            mov.pp + self.pp_ups[slot as usize] * (mov.pp / 5)
        } else {
            0
        }
    }

    fn stat_core(&self, stat: Stat) -> u16 {
        let base = self.pokemon.stats[stat] as u16;
        let iv = self.ivs[stat] as u16;
        let ev = self.evs[stat] as u16;
        let level = self.level as u16;
        (((2 * base) + iv + (ev / 4)) * level) / 100
    }

    pub fn stat(&self, stat: Stat) -> u16 {
        match stat {
            Stat::HP => self.stat_core(stat) + (self.level as u16) + 10,
            Stat::Accuracy | Stat::Evasion => 1,
            _ => {
                let core = self.stat_core(stat);
                if self.nature.increased().map_or(false, |s| s == stat) {
                    ((core + 5) * 11) / 10
                } else if self.nature.decreased().map_or(false, |s| s == stat) {
                    ((core + 5) * 9) / 10
                } else {
                    core + 5
                }
            },
        }
    }

    pub fn verify(&self, skip_moveset: bool) -> bool {
        let genderless = self.gender == pokemon::Gender::Genderless;
        let species_genderless = species_ref(self.pokemon.id).gender_rate < 0;
        let gender = genderless == species_genderless;
        let abilities = self.pokemon.abilities;
        let ability = abilities.first() == self.ability
            || abilities.second().map_or(false, |a| a == self.ability);
        let held = self.held.map_or(true,
            |item| item.flags.contains(items::Flags::HOLDABLE));
        let evs = self.evs.0.iter().fold(0, |acc, ev| acc + (*ev as u16)) <= 510;
        let ivs = !self.ivs.0.iter().any(|iv| *iv > 31);
        let pp_ups = self.pp_ups.iter().all(|ups| *ups <= 3);
        let level = self.level > 0 && self.level <= 100;
        gender && ability && held && evs && ivs && pp_ups && level
            && self.verify_moves(skip_moveset)
    }

    fn verify_moves(&self, skip_moveset: bool) -> bool {
        let mut list = self.moves.iter().filter_map(
            |opt| opt.map(|mov| mov.id)).collect::<Vec<_>>();
        list.sort_unstable();
        let count = list.len();
        list.dedup();
        count > 0 && list.len() == count && (skip_moveset || list.iter().all(
            |mov| cache_moveset(self.pokemon.id, |set| set.contains(&mov))))
    }
}

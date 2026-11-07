//! Activity: the stock shimmer that dresses an app's work under way.

use std::fmt;
use std::hash::{BuildHasher, Hash, RandomState};

use libshimmer::{Glyphs, Recipe, Shimmer, stock};

/// The glyphs an app assumes its terminal draws: the base set alone, since no terminal reports
/// which glyphs it draws.
pub const GLYPHS: Glyphs = Glyphs::BASE;

/// One stock recipe, picked at random for the process, and the seeds its shimmers grow from.
pub struct Ember {
    recipe: &'static Recipe,
    seeds: RandomState,
}

impl Ember {
    /// Picks a random stock recipe that draws in [`GLYPHS`] and does not listen, since a terminal
    /// app hears nothing; none only while libshimmer's roster offers no such recipe.
    pub fn kindle() -> Option<Self> {
        let seeds = RandomState::new();
        let fitting: Vec<&'static Recipe> = stock::ROSTER
            .iter()
            .filter(|recipe| recipe.fits(GLYPHS).is_ok() && !recipe.listens())
            .collect();
        let pick = usize::try_from(seeds.hash_one("recipe"))
            .ok()?
            .checked_rem(fitting.len())?;
        Some(Self {
            recipe: fitting.get(pick).copied()?,
            seeds,
        })
    }

    /// A fresh shimmer for the work `seed` names; within a process, one seed grows one shimmer.
    ///
    /// # Panics
    ///
    /// Never: [`Ember::kindle`] admits only recipes that draw in [`GLYPHS`].
    #[expect(
        clippy::expect_used,
        reason = "kindle admits only recipes that draw in GLYPHS"
    )]
    pub fn light(&self, seed: impl Hash) -> Shimmer {
        Shimmer::new(self.recipe, self.seeds.hash_one(seed), GLYPHS)
            .expect("a kindled recipe draws in GLYPHS")
    }
}

impl fmt::Debug for Ember {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Ember")
            .field("recipe", &self.recipe.name())
            .finish_non_exhaustive()
    }
}

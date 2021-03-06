//! The central part of this crate, uses all modules to load and run our world in memory.
//!
//! The `Board` struct is technically all you need to start your world but then you wouldn't be able to see it!
//! Graphics are provided by the [graphics] module; although you could implement your own.
//!
//! TODO: documentation.

extern crate bincode;
extern crate rand;
#[cfg(multithreading)]
extern crate rayon;

use crate::brain::{Brain, GenerateRandom, NeuralNet, RecombinationInfinite};
use crate::climate::Climate;
use crate::constants::*;
use crate::sbip::SoftBodiesInPositions;
use crate::softbody::{HLSoftBody, SoftBody};
use crate::terrain::Terrain;

/// The amount of times a year an object is updated.
///
/// TODO: eliminate this variable because it's not needed.
const OBJECT_TIMESTEPS_PER_YEAR: f64 = 100.0;
const _POPULATION_HISTORY_LENGTH: usize = 200;

pub type BoardSize = (usize, usize);
pub type BoardCoordinate = (usize, usize);
#[derive(Clone)]
pub struct BoardPreciseCoordinate(pub f64, pub f64);

impl BoardPreciseCoordinate {
    pub fn unpack(&self) -> (f64, f64) {
        return (self.0, self.1);
    }
}

impl From<BoardPreciseCoordinate> for BoardCoordinate {
    fn from(bpc: BoardPreciseCoordinate) -> BoardCoordinate {
        let (x, y) = bpc.unpack();

        (x.floor() as usize, y.floor() as usize)
    }
}

pub struct SelectedCreature<B: NeuralNet>(pub Option<HLSoftBody<B>>);

impl<B: NeuralNet> Default for SelectedCreature<B> {
    fn default() -> Self {
        SelectedCreature(None)
    }
}

impl<B: NeuralNet> SelectedCreature<B> {
    /// Checks if the given creature was selected and if so, removes it by setting `self.0` to `None`.
    pub fn unselect_if_dead(&mut self, creature: HLSoftBody<B>) {
        if let Some(sel_creature) = &self.0 {
            // If `creature` isn't the same as `self.selected_creature`.
            if *sel_creature != creature {
                // Then don't change to `None`.
                return;
            }

            // Else go on
        }

        self.0 = None;
    }

    pub fn select(&mut self, creature: HLSoftBody<B>) {
        self.0 = Some(creature);
    }

    pub fn deselect(&mut self) {
        self.0 = None;
    }
}

pub struct Board<B: NeuralNet = Brain> {
    // Fields relevant for the board itself.
    board_width: usize,
    board_height: usize,
    pub terrain: Terrain,

    // Fields relevant for the creatures.
    creature_minimum: usize,
    pub soft_bodies_in_positions: SoftBodiesInPositions<B>,
    pub creatures: Vec<HLSoftBody<B>>,
    creature_id_up_to: usize,
    // _creature_rank_metric: usize,

    // Fields relevant for time or history
    year: f64,

    // Fields relevant for temperature
    pub climate: Climate,

    // Miscelanious
    pub selected_creature: SelectedCreature<B>,
}

impl<B: NeuralNet + GenerateRandom> Default for Board<B> {
    fn default() -> Self {
        let board_size = DEFAULT_BOARD_SIZE;
        let noise_step_size = DEFAULT_NOISE_STEP_SIZE;
        let creature_minimum = DEFAULT_CREATURE_MINIMUM;
        let min_temp = DEFAULT_MIN_TEMP;
        let max_temp = DEFAULT_MAX_TEMP;

        return Board::new_random(
            board_size,
            noise_step_size,
            creature_minimum,
            min_temp,
            max_temp,
        );
    }
}

impl<B: NeuralNet> Board<B> {
    pub fn new(board_width: usize, board_height: usize, terrain: Terrain, creature_minimum: usize, soft_bodies_in_positions: SoftBodiesInPositions<B>,
    creatures: Vec<HLSoftBody<B>>, creature_id_up_to: usize, year: f64, climate: Climate, selected_creature: SelectedCreature<B>) -> Board<B>{
        Board {
            board_width,
            board_height,
            terrain,

            creature_minimum,
            soft_bodies_in_positions,
            creatures,
            creature_id_up_to,

            year,

            climate,

            selected_creature,
        }
    }
}

impl<B: NeuralNet + GenerateRandom> Board<B> {
    /// Randomly generates a new `Board`.
    pub fn new_random(
        board_size: BoardSize,
        noise_step_size: f64,
        creature_minimum: usize,
        min_temp: f64,
        max_temp: f64,
    ) -> Self {
        let creatures = Vec::with_capacity(creature_minimum);

        // Initialize climate.
        let mut climate = Climate::new(min_temp, max_temp);
        climate.update(0.0);

        let mut board = Board {
            board_width: board_size.0,
            board_height: board_size.1,
            terrain: Terrain::generate_perlin(board_size, noise_step_size),

            creature_minimum,
            soft_bodies_in_positions: SoftBodiesInPositions::new_allocated(board_size),
            creatures,
            creature_id_up_to: 0,

            year: 0.0,

            climate,

            selected_creature: SelectedCreature::default(),
        };

        // Initialize creatures.
        board.maintain_creature_minimum();

        return board;
    }

    /// Maintains the creature minimum by adding random creatures until there are at least `self.creature_minimum` creatures.
    ///
    /// # Processing equivalent
    /// This function is the equivalent of *Board.pde/maintainCreatureMinimum* with *choosePreexisting* set to false.
    fn maintain_creature_minimum(&mut self) {
        while self.creatures.len() < self.creature_minimum {
            let board_size = self.get_board_size();
            let creature = HLSoftBody::from(SoftBody::new_random(board_size, self.year));

            // Initialize in `SoftBodiesInPositions` as well.
            creature.set_sbip(&mut self.soft_bodies_in_positions, board_size);
            // Just to set the prevSBIP variables.
            creature.set_sbip(&mut self.soft_bodies_in_positions, board_size);

            self.creatures.push(creature);
            self.creature_id_up_to += 1;
        }
    }
}

impl<B: NeuralNet + RecombinationInfinite + GenerateRandom> Board<B> {
    pub fn update(&mut self, time_step: f64) {
        self.year += time_step;
        self.climate.update(self.year);

        let temp_change_into_frame =
            self.climate.get_temperature() - self.climate.get_growth_rate(self.year - time_step);
        let temp_change_out_of_frame =
            self.climate.get_growth_rate(self.year + time_step) - self.climate.get_temperature();

        if temp_change_into_frame * temp_change_out_of_frame < 0.0 {
            // Temperature change flipped direction
            self.terrain.update_all(self.year, &self.climate);
        }

        self.update_creatures(time_step);

        // Kill weak creatures.
        self.remove_dead_creatures();

        // Let creatures reproduce
        self.creatures_reproduce();

        // Experimental: this was moved from above to always keep the creature minimum.
        self.maintain_creature_minimum();

        // Move the creatures around on the board
        self.move_creatures(time_step);
    }
}

impl<B: NeuralNet + RecombinationInfinite> Board<B> {
    fn creatures_reproduce(&mut self) {
        let mut babies = Vec::new();

        // Keep the borrow checker happy
        {
            let time = self.get_time();
            let board_size = self.get_board_size();
            let sbip = &mut self.soft_bodies_in_positions;

            for c in &mut self.creatures {
                let maybe_baby = c.try_reproduce(time, sbip, board_size);
                if let Some(baby) = maybe_baby {
                    babies.push(baby);
                }
            }
        }

        babies.into_iter().for_each(|c| self.creatures.push(c));
    }
}

impl<B: NeuralNet> Board<B> {
    /// Selects the oldest creature still alive.
    pub fn select_oldest(&mut self) {
        let oldest = self.creatures.iter().fold(&self.creatures[0], |c_old, c| {
            if c.borrow().get_birth_time() < c_old.borrow().get_birth_time() {
                &c
            } else {
                c_old
            }
        });

        self.selected_creature.select(oldest.clone());
    }

    /// Selects the biggest creature.
    pub fn select_biggest(&mut self) {
        let biggest = self.creatures.iter().fold(&self.creatures[0], |c_old, c| {
            if c.borrow().get_energy() > c_old.borrow().get_energy() {
                &c
            } else {
                c_old
            }
        });

        self.selected_creature.select(biggest.clone());
    }

    #[cfg(not(multithreading))]
    fn update_brains(&mut self) {
        self.creatures
            .iter()
            .map(|c| c.borrow_mut())
            .for_each(|mut c| {
                let creature: &mut SoftBody<B> = &mut c;
                let env = crate::brain::Environment::new(&self.terrain, &creature.base);
                creature.brain.run_with(&env);
            });
    }

    #[cfg(multithreading)]
    fn update_brains(&mut self) {
        self.creatures
            .map(|c| c.borrow_mut())
            .par_iter()
            .for_each(|c| {
                let env = crate::brain::Environment::new(&self.terrain, &c.base);
                c.brain.run_with(&env);
            });
    }

    pub fn update_creatures(&mut self, time_step: f64) {
        use crate::brain::EnvironmentMut;

        let time = self.year;
        let board_size = self.get_board_size();

        for c_rc in &self.creatures {
            // These functions call `borrow_mut()`
            c_rc.collide(&self.soft_bodies_in_positions);

            let mut c = c_rc.borrow_mut();

            c.record_energy();

            c.metabolize(time_step, time);
        }

        self.update_brains();

        let use_output = true;
        if use_output {
            for c_rc in &self.creatures {
                let creature: &mut SoftBody<B> = &mut c_rc.borrow_mut();
                let mut env = EnvironmentMut::new(
                    &mut self.terrain,
                    &mut creature.base,
                    board_size,
                    time,
                    &self.climate,
                    &self.soft_bodies_in_positions,
                    c_rc.clone(),
                );
                creature.brain.use_output(&mut env, time_step);
            }
        }
    }

    // #[cfg(multithreading)]
    pub fn move_creatures(&mut self, time_step: f64) {
        let board_size = self.get_board_size();

        for c in &self.creatures {
            c.apply_motions(
                time_step * OBJECT_TIMESTEPS_PER_YEAR,
                board_size,
                &self.terrain,
                &mut self.soft_bodies_in_positions,
            );
        }
    }

    pub fn prepare_for_drawing(&mut self) {
        self.terrain.update_all(self.year, &self.climate);
    }

    /// Checks for all creatures whether they are fit enough to live and kills them off if they're not.
    ///
    /// Utilizes the `should_die` function of `SoftBody`.
    fn remove_dead_creatures(&mut self) {
        let time = self.get_time();
        let board_size = self.get_board_size();
        let terrain = &mut self.terrain;
        let climate = &self.climate;
        let sbip = &mut self.soft_bodies_in_positions;

        // TODO: possibly optimise code
        let mut i = 0;
        while i < self.creatures.len() {
            // let creature = &mut self.creatures[i];
            if self.creatures[i].borrow().should_die() {
                self.creatures[i].return_to_earth(time, board_size, terrain, climate, sbip);

                self.selected_creature
                    .unselect_if_dead(self.creatures[i].clone());
                self.creatures.remove(i);

            // println!("Dead!");
            } else {
                i += 1;
            }
        }
    }

    /// Performs the same function on `self.climate`, filling in `self.year`.
    pub fn get_growth_since(&self, last_updated: f64) -> f64 {
        return self
            .climate
            .get_growth_over_time_range(self.year, last_updated);
    }

    /// Returns the current growth rate (temperature) based on the season.
    ///
    /// Performs the same function on `self.climate`, filling in `self.year`.
    pub fn get_current_growth_rate(&self) -> f64 {
        self.climate.get_growth_rate(self.year)
    }

    /// Returns the current time, i.e. `self.year`.
    pub fn get_time(&self) -> f64 {
        return self.year;
    }

    /// Returns a tuple with the width and height of this `Board`.
    ///
    /// Equivalent to `(board.get_board_width(), board.get_board_height())`.
    pub fn get_board_size(&self) -> (usize, usize) {
        return (self.board_width, self.board_height);
    }

    /// Returns the width of the board.
    pub fn get_board_width(&self) -> usize {
        return self.board_width;
    }

    /// Returns the height of the board.
    pub fn get_board_height(&self) -> usize {
        return self.board_height;
    }

    /// Returns the minimum amount of creatures that should be on the `Board`
    /// 
    /// When the population drops below this `maintain_creature_minimum()` spawns new creatures to fill the gap.
    pub fn get_creature_minimum(&self) -> usize {
        self.creature_minimum
    }
    
    /// Returns `self.creature_id_up_to`
    pub fn get_creature_id_up_to(&self) -> usize {
        self.creature_id_up_to
    }

    /// Gets the size of the current population; i.e. how many creatures are currently alive.
    pub fn get_population_size(&self) -> usize {
        return self.creatures.len();
    }

    /// Returns a `String` representing the current season.
    ///
    /// Can be either "Winter", "Spring", "Summer" or "Autumn".
    pub fn get_season(&self) -> String {
        const SEASONS: [&str; 4] = ["Winter", "Spring", "Summer", "Autumn"];
        let season: usize = ((self.year % 1.0) * 4.0).floor() as usize;

        return SEASONS[season].to_string();
    }
}

impl<B: NeuralNet + serde::de::DeserializeOwned> Board<B> {
    pub fn load_from<P: AsRef<std::path::Path>>(path: P) -> Result<Board<B>, Box<std::error::Error>> {
        let file = std::fs::File::open(path)?;
        Ok({
            use crate::serde_structs::board::BoardSerde;
            let ir: BoardSerde<B> = bincode::deserialize_from(file)?;
            
            ir.into()
        })
    }
}

impl<B: NeuralNet + serde::Serialize> Board<B> {
    pub fn save_to<P: AsRef<std::path::Path>>(
        self,
        path: P,
    ) -> Result<(), Box<std::error::Error>> {
        let file = std::fs::File::create(path)?;
        bincode::serialize_into(file, &crate::serde_structs::board::BoardSerde::from(self))?;

        Ok(())
    }
}

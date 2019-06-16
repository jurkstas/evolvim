use crate::board::BoardSize;
use crate::brain::Brain;
use crate::constants;
use crate::softbody::Creature;

pub struct ECSBoard {
    world: specs::World,
}

impl ECSBoard {
    pub fn init(board_size: BoardSize, noise_step_size: f64) -> Self {
        let mut world = specs::World::new();

        // Register components
        world.register::<Creature<Brain>>();

        // Initialise resources
        let terrain = crate::terrain::Terrain::generate_perlin(board_size, noise_step_size);
        world.add_resource(terrain);

        let (min_temp, max_temp) = (constants::DEFAULT_MIN_TEMP, constants::DEFAULT_MAX_TEMP);
        let climate = crate::climate::Climate::new(min_temp, max_temp);
        world.add_resource(climate);

        let mut physics_world = nphysics2d::world::World::<f64>::new();
        physics_world.set_timestep(0.001);
        world.add_resource(physics_world);

        let time = crate::time::Time::default();
        world.add_resource(time);

        world.add_resource(board_size);

        // Return the world
        ECSBoard { world }
    }

    pub fn run(&mut self) {
        use specs::RunNow;
        use crate::systems::*;

        let mut res_up = UpdateResources;
        let mut creat_up = UpdateCreatures;
        let mut rm_dead_creat = RemoveDeadCreatures;
        let mut creat_rep = CreaturesReproduce;
        let mut refill_creat = RefillCreatures;
        let mut physics = PhysicsStep;

        res_up.run_now(&mut self.world.res);
        creat_up.run_now(&mut self.world.res);
        rm_dead_creat.run_now(&mut self.world.res);
        creat_rep.run_now(&mut self.world.res);
        refill_creat.run_now(&mut self.world.res);
        physics.run_now(&mut self.world.res);

        // Synchronize deletions and insertions
        self.world.maintain();
    }

    pub fn get_time(&self) -> f64 {
        self.world.read_resource::<crate::time::Time>().0
    }

    /// Returns a `String` representing the current season.
    ///
    /// Can be either "Winter", "Spring", "Summer" or "Autumn".
    pub fn get_season(&self) -> String {
        const SEASONS: [&str; 4] = ["Winter", "Spring", "Summer", "Autumn"];
        let season: usize = ((self.get_time() % 1.0) * 4.0).floor() as usize;

        return SEASONS[season].to_string();
    }

    pub fn get_population_size(&self) -> usize {
        use specs::Join;

        self.world.read_storage::<Creature<Brain>>().join().count()
    }

    pub fn get_board_size(&self) -> BoardSize {
        *self.world.read_resource::<BoardSize>()
    }

    pub fn get_board_width(&self) -> usize {
        self.get_board_size().0
    }

    pub fn get_board_height(&self) -> usize {
        self.get_board_size().1
    }
}
extern crate rand;

mod gene;
mod recombination;
mod mutation;
// mod innovations;
mod speciation;
mod utils;

use self::gene::{NodeGene, ConnectionGene, Id, NodeType};
use rand::Rng;

const AMOUNT_INPUT: usize = 3;
const AMOUNT_OUTPUT: usize = 2;
static mut INNOVATION_NUMBER: usize = AMOUNT_INPUT * AMOUNT_OUTPUT;
static mut NODE_NUMBER: Id = AMOUNT_INPUT + AMOUNT_OUTPUT;

fn get_innovation_number() -> usize {
    unsafe {
        INNOVATION_NUMBER += 1;
        return INNOVATION_NUMBER;
    }
}

pub fn get_next_node_id() -> Id {
    unsafe {
        NODE_NUMBER += 1;
        return NODE_NUMBER;
    }
}

pub struct Genome {
    node_genome: Vec<NodeGene>,
    connection_genome: Vec<ConnectionGene>,
}

impl Genome {
    fn get_random_node_id(&self) -> Id {
        self.node_genome[self.get_random_node_place()].id
    }

    fn get_random_node_place(&self) -> usize {
        rand::thread_rng().gen_range(0, self.node_genome.len())
    }

    fn get_random_connection_place(&self) -> usize {
        rand::thread_rng().gen_range(0, self.connection_genome.len())
    }

    fn get_random_weight() -> f64 {
        rand::random::<f64>() * 2.0 - 1.0
    }

    fn get_random_weight_multiplier() -> f64 {
        rand::random::<f64>() * 0.4 + 0.8
    }

    fn add_node(&mut self, node_type: NodeType, id: Id) {
        self.node_genome.push(NodeGene {
            node_type,
            id,
        });
    }

    fn add_connection(&mut self, from: Id, to: Id, weight: f64) {
        self.connection_genome.push(ConnectionGene {
            from,
            to,
            weight,

            enabled: true,
            innovation_number: get_innovation_number(),
        });
    }

    pub fn new_fully_linked() -> Self {
        let mut genome = Genome {
            node_genome: Vec::new(),
            connection_genome: Vec::new(),
        };
        let mut node_counter = 1;

        for _i in 0..AMOUNT_INPUT {
            genome.add_node(NodeType::Sensor, node_counter);
            node_counter += 1;
        }

        let mut con_counter = 1;
        for _i in 0..AMOUNT_OUTPUT {
            genome.add_node(NodeType::Output, node_counter);
            node_counter += 1;
            
            let to = genome.node_genome.last().unwrap().id;
            for i in 0..AMOUNT_INPUT {
                let from = genome.node_genome[i].id;

                // Because all creatures start with this basic genome give all the connections the same innovation number
                // `counter` is used for this purpose
                genome.connection_genome.push(ConnectionGene {
                    from,
                    to,
                    weight: Self::get_random_weight(),

                    enabled: true,
                    innovation_number: con_counter,
                });
                con_counter += 1;
            }
        }

        return genome;
    }
}

impl Genome {
    pub fn log_nodes(&self) {
        for n in &self.node_genome {
            println!("\tnode {} is {:?}", n.id, n.node_type);
        }
    }

    pub fn log_connections(&self) {
        for n in &self.connection_genome {
            print!("\t");
            if n.enabled == false {
                print!("DISABLED! ");
            }
            println!("innovation {}: from {} to {} with weight {}", n.innovation_number, n.from, n.to, n.weight);
        }
    }
}
extern crate conv;
extern crate rand;

use self::conv::prelude::*;
use neat::gene::Gene as Gene;
use neat::mutation::Mutation as Mutation;


#[derive(Debug, Clone)]
pub struct Genome{
    genes: Vec<Gene>,
    last_neuron_id: u32
}

const COMPATIBILITY_THRESHOLD: f64 = 1f64;
//const MUTATE_ADD_NEURON_PROBABILITY: f64 = 0.33f64;
const MUTATE_CONNECTION_WEIGHT: f64 = 0.33f64;
const MUTATE_ADD_CONNECTION: f64 = 0.33f64;

impl Genome{

    pub fn new() -> Genome {
        Genome { 
            genes: vec![],
            last_neuron_id: 0 //we need 1 neuron to start to mutate
        }
    }

    pub fn mutate(&mut self) {
        let random = rand::random::<f64>();
        if random > MUTATE_ADD_CONNECTION + MUTATE_CONNECTION_WEIGHT && self.genes.len() > 0 {
            self.mutate_add_neuron();
        } else if random > MUTATE_ADD_CONNECTION && self.genes.len() > 0 {
            self.mutate_connection_weight();
        } else {
            self.mutate_add_connection();
        }
    }

    pub fn mate(&self, other: &Genome) -> Genome{
        if self.genes.len() > other.genes.len() {
            self.mate_genes(other)    
        }else{
            other.mate_genes(self)
        }
    }

    fn mate_genes(&self, other: &Genome) -> Genome{
        let mut genome = Genome::new();
        for gene in &self.genes {
            genome.genes.push({
                if rand::random::<f64>() > 0.5f64 {
                    match other.genes.binary_search(&gene) {
                        Ok(position) => other.genes[position].clone(),
                        Err(_) => gene.clone() 
                    }
                } else {
                    gene.clone()
                }
            });
        }
        genome
    }

    // call this method directly can create non connected neurons
    pub fn inject_gene(&mut self, in_neuron_id: u32, out_neuron_id: u32, weight: f64) {
        if in_neuron_id > self.last_neuron_id {
            self.last_neuron_id = in_neuron_id;
        }
        if out_neuron_id > self.last_neuron_id {
            self.last_neuron_id = out_neuron_id;
        }
        self.create_gene(in_neuron_id, out_neuron_id, weight)
    }

    fn create_gene(&mut self, in_neuron_id: u32, out_neuron_id: u32, weight: f64) {
        let gene = Gene {
            in_neuron_id: in_neuron_id,
            out_neuron_id: out_neuron_id,
            weight: weight,
            ..Default::default()
        };
        self.genes.push(gene);
    }

    fn mutate_add_connection(&mut self) {
        let mut rng = rand::thread_rng();
        let connections = {
            if self.last_neuron_id == 0 {
                vec![0, 0]
            } else {
                rand::sample(&mut rng, 0..self.last_neuron_id + 1, 2)
            }
        };
        self.add_connection(connections[0], connections[1]);
    }

    fn mutate_connection_weight(&mut self) {
        let mut rng = rand::thread_rng();
        let selected_gene = rand::sample(&mut rng, 0..self.genes.len(), 1)[0];
        let gene = &mut self.genes[selected_gene];
        Mutation::connection_weight(gene);
    }

    fn mutate_add_neuron(&mut self) {
        let (gene1, gene2) = {
            let mut rng = rand::thread_rng();
            let selected_gene = rand::sample(&mut rng, 0..self.genes.len(), 1)[0];
            let gene = &mut self.genes[selected_gene];
            self.last_neuron_id += 1;
            Mutation::add_neuron(gene, self.last_neuron_id)
        };
        self.genes.push(gene1);
        self.genes.push(gene2);
    }

    fn add_connection(&mut self, in_neuron_id: u32, out_neuron_id: u32) {
        let gene = Mutation::add_connection(in_neuron_id, out_neuron_id);
        self.genes.push(gene);
    }

    pub fn is_same_specie(&self, other: &Genome) -> bool{
        self.compatibility_distance(other) < COMPATIBILITY_THRESHOLD
    }

    pub fn total_weights(&self) -> f64{
        let mut total = 0f64;
        for gene in &self.genes {
            total += gene.weight;
        }
        total
    }

    pub fn total_genes(&self) -> usize{
        self.genes.len()
    }

    //http://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf - Pag. 110
    //I have consider disjoint and excess genes as the same
    fn compatibility_distance(&self, other: &Genome) -> f64 {
        //TODO: optimize this method
        let c2 = 0.5f64;
        let c3 = 0.5f64;

        //Number of excess
        let n1 = self.genes.len().value_as::<f64>().unwrap();
        let n2 = other.genes.len().value_as::<f64>().unwrap();
        let n = n1.max(n2);

        if n == 0f64 {
            return 0f64; //no genes in any genome, the genomes are equal
        }

        let matching_genes  = self.genes.iter().filter(|i1_gene| other.genes.contains(i1_gene)).collect::<Vec<&Gene>>();

        let n3 = matching_genes.len().value_as::<f64>().unwrap();

        //Disjoint genes
        let d = n1 + n2 - (2f64 * n3);

        //average weight differences of matching genes
        let w1 = matching_genes.iter().fold(0f64, |acc, &m_gene| acc + (m_gene.weight + other.genes.get(other.genes.binary_search(m_gene).unwrap()).unwrap().weight)).abs();

        let w = w1 / n3;

        //compatibility distance
        let delta = (c2 * d / n) + c3 * w;
        delta
    }
}

#[cfg(test)]
mod tests {
    use neat::*;

    #[test]
    fn mutation_connection_weight(){
        let mut genome = Genome::new();
        genome.create_gene(1, 1, 1f64);
        let orig_gene = genome.genes[0].clone();
        genome.mutate_connection_weight();

        assert!(genome.genes[0].weight != orig_gene.weight);
    }

    #[test]
    fn mutation_add_connection(){
        let mut genome = Genome::new();
        genome.add_connection(1, 2);
        
        assert!(genome.genes[0].in_neuron_id == 1);
        assert!(genome.genes[0].out_neuron_id == 2);
    }

    #[test]
    fn mutation_add_neuron(){
        let mut genome = Genome::new();
        genome.mutate_add_connection();
        genome.mutate_add_neuron();
        assert!(!genome.genes[0].enabled);
        assert!(genome.genes[1].in_neuron_id == genome.genes[0].in_neuron_id);
        assert!(genome.genes[1].out_neuron_id == 1);
        assert!(genome.genes[2].in_neuron_id == 1);
        assert!(genome.genes[2].out_neuron_id == genome.genes[0].out_neuron_id);
    }

    #[test]
    fn two_genomes_without_differences_should_be_in_same_specie(){
        let mut genome1 = Genome::new();
        genome1.create_gene(1, 1, 1f64);
        genome1.create_gene(1, 2, 1f64);
        let mut genome2 = Genome::new();
        genome2.create_gene(1, 1, 0f64);
        genome2.create_gene(1, 2, 0f64);
        genome2.create_gene(1, 3, 0f64);
        assert!(genome1.is_same_specie(&genome2));
    }

    #[test]
    fn two_genomes_with_enought_difference_should_be_in_different_species(){
        let mut genome1 = Genome::new();
        genome1.create_gene(1, 1, 1f64);
        genome1.create_gene(1, 2, 1f64);
        let mut genome2 = Genome::new();
        genome2.create_gene(1, 3, 1f64);
        genome2.create_gene(1, 4, 1f64);
        assert!(!genome1.is_same_specie(&genome2));
    }
}
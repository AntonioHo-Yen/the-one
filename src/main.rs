pub mod authority;
pub mod crypto;
pub mod egress;
pub mod state; // 🟢 Points to src/state/mod.rs
pub mod error;
use crate::state::evaluator::Evaluator; // Import Evaluator from its new state home

fn main() {
    println!("Evaluator engine initialized.");
    
    // Example: Create an instance or invoke a method
    let _evaluator = Evaluator; 
}
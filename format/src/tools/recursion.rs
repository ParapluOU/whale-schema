use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Making IdentifierCounter globally accessible using Arc<Mutex<>>
lazy_static::lazy_static! {
    static ref IDENTIFIER_COUNTER: Arc<Mutex<IdentifierCounter>> = Arc::new(Mutex::new(IdentifierCounter::new()));
}

// Define a struct to hold the state of the identifier counts
pub struct IdentifierCounter {
    counts: HashMap<String, usize>,
}

impl IdentifierCounter {
    // Function to create a new instance of IdentifierCounter
    pub fn new() -> IdentifierCounter {
        IdentifierCounter {
            counts: HashMap::new(),
        }
    }

    // Function to increment count for the given identifier
    pub fn increment(&mut self, identifier: &str) {
        let count = self.counts.entry(identifier.to_string()).or_insert(0);
        *count += 1;
    }

    // Function to check if the identifier has reached the given count
    pub fn check_and_panic(&mut self, identifier: &str, nth: usize) {
        self.increment(identifier);

        if *self.counts.get(identifier).unwrap() >= nth {
            panic!(
                "Identifier '{}' reached its maximum count of {}",
                identifier, nth
            );
        }
    }
}

pub fn panic_nth(identifier: &str, nth: usize) {
    IDENTIFIER_COUNTER
        .lock()
        .unwrap()
        .check_and_panic(identifier, nth);
}

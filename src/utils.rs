use rand::prelude::IndexedRandom;
use std::time::Duration;

pub fn random_delay() {
    let delays = [1000, 1500, 2000];
    let delay = delays.choose(&mut rand::rng()).unwrap();
    std::thread::sleep(Duration::from_millis(*delay));
}

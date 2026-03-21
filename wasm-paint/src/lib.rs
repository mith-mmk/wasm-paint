mod utils;

use wasm_bindgen::prelude::*;
pub mod universe;
pub use universe::Universe;
// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn initialization() {
    utils::set_panic_hook();
}



pub struct Rnd {
    seed: u64,
}

impl Rnd {
    pub fn new() -> Self {
        let seed: u64 = instant::Instant::now().elapsed().as_nanos() as u64;
        Self { seed }
    }

    pub fn get_u32(&mut self, range: u32) -> u32 {
        let mut seed = self.seed;
        // xorshift64
        seed = seed ^ (seed >> 13);
        seed = seed ^ (seed << 11);
        self.seed = seed ^ (seed >> 8);

        (seed % range as u64) as u32
    }
}


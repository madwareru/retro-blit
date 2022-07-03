use std::time::Instant;

pub struct StopWatch {
    instant: Instant,
    name: &'static str
}

impl StopWatch {
    pub fn named(name: &'static str) -> Self {
        Self { name, instant: Instant::now() }
    }
}

impl Drop for StopWatch {
    fn drop(&mut self) {
        println!("{}: {} ms", self.name, self.instant.elapsed().as_secs_f32() * 1000.0)
    }
}
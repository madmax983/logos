#[cfg(feature = "nova")]
pub struct NarrativeGenerator;

#[cfg(feature = "nova")]
impl NarrativeGenerator {
    pub fn new() -> Self {
        Self
    }
}

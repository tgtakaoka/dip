use crate::cli::{AltNames, Direction, PinGap, Side};

/// Trait for IC package types (DIP, QFP, PLCC, etc.)
pub trait Package {
    /// Get the IC name
    fn name(&self) -> &str;
    
    /// Get the IC title/description
    fn title(&self) -> &str;
    
    /// Get the total pin count
    fn pin_count(&self) -> usize;
    
    /// Print the package visualization
    fn print(
        &self,
        dir: Direction,
        side: Side,
        show_pin: PinGap,
        show_alt: AltNames,
    ) -> Vec<String>;
}

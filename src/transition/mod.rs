mod transition;

pub use transition::*;


/// Common transition status that prevents multiple transitioners from running at the same time.
#[derive(Default)]
pub enum TransitionState {

    /// Not transitioning
    #[default]
    Idle,

    /// First half of transition (IE, fades to black)
    FirstHalf,

    /// Middle of transition. Waiting on event to continue (IE, screen remains black for a certain amount of time)
    Waiting,

    /// Event received, transition finishing. (IE, fades from black back to normal)
    SecondHalf
}
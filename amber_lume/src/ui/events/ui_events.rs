#[derive(Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Forward,
    Back,
}

#[derive(Debug, Eq, PartialEq)]
pub enum EventState {
    Down,
    Up,
}

#[derive(Debug)]
pub enum MouseEvent {
    Move {
        position: [f32; 2],
    },
    Click {
        button: MouseButton,
        state: EventState,
    },
    Scroll {
        delta: [f32; 2],
    }
}

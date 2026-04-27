use yakui::event::{EventInterest, EventResponse, WidgetEvent};
use yakui::input::MouseButton;
use yakui::util::widget_children;
use yakui::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui::{Constraints, Response, Vec2};

#[derive(Debug)]
pub struct Clickable;

impl Clickable {
    pub fn new() -> Self {
        Self
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<ClickableResponse> {
        widget_children::<ClickableWidget, _>(children, self)
    }
}

#[derive(Debug)]
pub struct ClickableWidget {
    hovering: bool,
    pressed: bool,
    clicked: bool,
}

#[derive(Debug)]
pub struct ClickableResponse {
    pub clicked: bool,
    pub hovering: bool,
    pub pressed: bool,
}

impl Widget for ClickableWidget {
    type Props<'widget> = Clickable;
    type Response = ClickableResponse;

    fn new() -> Self {
        Self {
            hovering: false,
            pressed: false,
            clicked: false,
        }
    }

    fn update(&mut self, _props: Self::Props<'_>) -> Self::Response {
        let clicked = self.clicked;
        self.clicked = false;

        ClickableResponse {
            clicked,
            hovering: self.hovering,
            pressed: self.pressed,
        }
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();

        let mut size = Vec2::ZERO;

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, constraints);

            size = size.max(child_size);
            ctx.layout.set_pos(child, Vec2::ZERO);
        }

        constraints.constrain(size)
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let node = ctx.dom.get_current();

        for &child in &node.children {
            ctx.paint(child);
        }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE | EventInterest::MOUSE_OUTSIDE
    }

    fn event(&mut self, _ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match *event {
            WidgetEvent::MouseEnter => {
                self.hovering = true;

                EventResponse::Bubble
            }
            WidgetEvent::MouseLeave => {
                self.hovering = false;
                self.pressed = false;

                EventResponse::Bubble
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: true,
                inside: true,
                ..
            } => {
                self.pressed = true;

                EventResponse::Sink
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: false,
                inside,
                ..
            } => {
                let was_down = self.pressed;
                self.pressed = false;

                if was_down && inside {
                    self.clicked = true;

                    EventResponse::Sink
                } else {
                    EventResponse::Bubble
                }
            }
            _ => EventResponse::Bubble,
        }
    }
}

pub fn clickable<F: FnOnce()>(children: F) -> Response<ClickableResponse> {
    Clickable::new().show(children)
}

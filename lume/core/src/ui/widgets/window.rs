use std::cell::Cell;
use yakui::event::{EventInterest, EventResponse, WidgetEvent};
use yakui::{expanded, pad, scroll_vertical, Alignment, Color, Constraints, Dim2, Flow, Response, Vec2};
use yakui::input::MouseButton;
use yakui::util::widget_children;
use yakui::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui::widgets::{Pad, Text};
use ui::Theme;
use crate::ui::widgets::paint_utils::draw_rect;

#[derive(Debug)]
pub struct Window<'widget> {
    title: &'widget str,

    position: Vec2,
    size: Vec2,

    style: WindowStyle,
}

#[derive(Debug)]
pub struct WindowStyle {
    pub header_font_size: f32,
    pub border_width: f32,

    pub header_background: Color,
    pub header_text: Color,

    pub background: Color,
    pub border: Color,
}

impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            header_font_size: 20.0,
            border_width: 1.0,

            header_background: Color::CLEAR,
            header_text: Color::rgba(255, 255, 255, 255),

            background: Color::CLEAR,
            border: Color::CLEAR,
        }
    }
}

impl<'widget> Window<'widget> {
    pub fn new(theme: &Theme, title: &'widget str) -> Self {
        Self {
            title,

            position: Vec2::ZERO,
            size: Vec2::new(660.0, 440.0),

            style: WindowStyle {
                header_font_size: 20.0,
                border_width: 1.0,

                header_background: theme.surface,
                header_text: Color::rgba(255, 255, 255, 255),

                background: theme.background,
                border: theme.surface,
            },
        }
    }

    pub fn position(mut self, position: Vec2) -> Self {
        self.position = position;

        self
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;

        self
    }

    pub fn show(self, children: impl FnOnce()) -> Response<WindowResponse> {
        let title = self.title.to_string();
        let title_font_size = self.style.header_font_size;
        let title_color = self.style.header_text;

        widget_children::<WindowWidget, _>(|| {
            pad(Pad::balanced(12.0, 6.0), || {
                let mut text = Text::new(title_font_size, title);
                text.style.color = title_color;
                text.show();
            });

            expanded(|| {
                scroll_vertical(children);
            });
        }, self)
    }
}

#[derive(Debug)]
pub struct WindowWidget {
    position: Vec2,
    size: Vec2,

    style: WindowStyle,

    initialized: bool,

    grip_size: Vec2,

    drag_offset: Option<Vec2>,
    resize_start: Option<(Vec2, Vec2)>,

    top_bar_height: Cell<f32>,
    viewport_size: Cell<Vec2>,
}

#[derive(Debug)]
pub struct WindowResponse;

impl WindowWidget {
    fn invalidated_position(&self, offset: Vec2, position: Vec2) -> Vec2 {
        let raw_position = position + offset;
        let min_position = Vec2::ZERO;
        let max_position = self.viewport_size.get() - self.size;

        raw_position.min(max_position).max(min_position)
    }

    fn invalidated_size(&self, offset: Vec2, position: Vec2) -> Vec2 {
        let raw_size = offset + position;
        let min_size = Vec2::new(100.0, 100.0);
        let max_size = self.viewport_size.get() - self.position;

        raw_size.min(max_size).max(min_size)
    }
}

impl Widget for WindowWidget {
    type Props<'widget> = Window<'widget>;
    type Response = WindowResponse;

    fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            size: Vec2::ZERO,

            style: WindowStyle::default(),

            initialized: false,

            grip_size: Vec2::splat(16.0),

            drag_offset: None,
            resize_start: None,

            top_bar_height: Cell::new(0.0),
            viewport_size: Cell::new(Vec2::ZERO),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        if !self.initialized {
            self.position = props.position;
            self.size = props.size;
            self.style = props.style;

            self.initialized = true;
        }

        Self::Response {}
    }

    fn flow(&self) -> Flow {
        let position = self.invalidated_position(Vec2::ZERO, self.position);

        Flow::Relative {
            anchor: Alignment::TOP_LEFT,
            offset: Dim2::pixels(position.x, position.y),
        }
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, _constraints: Constraints) -> Vec2 {
        ctx.layout.new_layer(ctx.dom);
        ctx.layout.enable_clipping(ctx.dom);

        let node = ctx.dom.get_current();
        let mut top_bar_height = 0.0;

        if let Some(&id) = node.children.get(0) {
            let constraints = Constraints {
                min: Vec2::ZERO,
                max: Vec2::new(self.size.x, f32::INFINITY),
            };

            let top_bar_size = ctx.calculate_layout(id, constraints);

            top_bar_height = top_bar_size.y;
            ctx.layout.set_pos(id, Vec2::ZERO);
        }

        let content_height = (self.size.y - top_bar_height).max(0.0);
        let content_constraints = Constraints {
            min: Vec2::ZERO,
            max: Vec2::new(self.size.x, content_height),
        };

        for &id in node.children.iter().skip(1) {
            ctx.calculate_layout(id, content_constraints);
            ctx.layout.set_pos(id, Vec2::new(0.0, top_bar_height));
        }

        self.top_bar_height.set(top_bar_height);
        self.viewport_size.set(ctx.layout.viewport().size());

        self.size
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();
        let top_bar_height = self.top_bar_height.get();
        let border_width = self.style.border_width;

        let position = layout_node.rect.pos();
        let size = layout_node.rect.size();

        let header_size = Vec2::new(size.x, top_bar_height);
        draw_rect(
            &mut ctx,
            position,
            header_size,
            self.style.header_background,
        );

        let border_position = position + Vec2::new(0.0, header_size.y);
        let border_size = size - Vec2::new(0.0, header_size.y);
        draw_rect(
            &mut ctx,
            border_position,
            border_size,
            self.style.border,
        );

        let content_background_position = border_position + border_width;
        let content_background_size = border_size - border_width * 2.0;
        draw_rect(
            &mut ctx,
            content_background_position,
            content_background_size,
            self.style.background,
        );

        let node = ctx.dom.get_current();
        for &child in &node.children {
            ctx.paint(child);
        }

        let grip_position = position + size - border_width - self.grip_size;
        let grip_size = self.grip_size;
        draw_rect(
            &mut ctx,
            grip_position,
            grip_size,
            Color::RED,
        );
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE | EventInterest::MOUSE_OUTSIDE | EventInterest::MOUSE_MOVE
    }

    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match *event {
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down,
                inside,
                position,
                ..
            } => {
                if inside && down {
                    let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
                    let local = position - rect.pos();

                    let top_bar_height = self.top_bar_height.get();

                    if local.y < top_bar_height {
                        self.drag_offset = Some(rect.pos() - position);
                    } else if local.x > self.size.x - self.grip_size.x && local.y > self.size.y - self.grip_size.y {
                        self.resize_start = Some((position, self.size));
                    }

                    EventResponse::Sink
                } else if !down {
                    self.drag_offset = None;
                    self.resize_start = None;

                    EventResponse::Sink
                } else {
                    EventResponse::Bubble
                }
            }
            WidgetEvent::MouseMoved(Some(position)) => {
                if let Some(offset) = self.drag_offset {
                    self.position = self.invalidated_position(offset, position);

                    EventResponse::Sink
                } else if let Some((mouse_start, size_start)) = self.resize_start {
                    self.size = self.invalidated_size(size_start - mouse_start, position);

                    EventResponse::Sink
                } else {
                    EventResponse::Bubble
                }
            }
            _ => EventResponse::Bubble,
        }
    }
}

pub fn window<F: FnOnce()>(theme: &Theme, title: &str, children: F) -> Response<WindowResponse> {
    Window::new(&theme, title).show(children)
}

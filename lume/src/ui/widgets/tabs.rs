use std::cell::{Cell, RefCell};
use yakui::{pad, Color, Constraints, Rect, Response, Vec2};
use yakui::event::{EventInterest, EventResponse, WidgetEvent};
use yakui::input::MouseButton;
use yakui::util::widget_children;
use yakui::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui::widgets::{Pad, Text};
use amber_lume::ui::theme::Theme;
use crate::ui::widgets::paint_utils::draw_rect;

#[derive(Debug)]
pub struct Tabs {
    tab_count: u32,

    selected: u32,

    style: TabsStyle,
}

#[derive(Debug)]
pub struct TabsStyle {
    pub font_size: f32,
    pub font_color: Color,

    pub tab_background: Color,

    pub tab_offset: f32,
}

impl Default for TabsStyle {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            font_color: Color::rgba(255, 255, 255, 255),

            tab_background: Color::rgba(50, 50, 55, 255),

            tab_offset: 2.0,
        }
    }
}

impl Tabs {
    pub fn new(theme: &Theme, selected: u32) -> Self {
        Self {
            tab_count: 0,

            selected,

            style: TabsStyle {
                tab_background: theme.surface,

                ..Default::default()
            },
        }
    }

    pub fn show(self, tabs: &[(&str, &dyn Fn())]) -> Response<TabsResponse> {
        let titles = tabs.iter().map(|(t, _)| t.to_string()).collect::<Vec<_>>();
        let content = tabs.iter().map(|(_, f)| *f).collect::<Vec<_>>();

        let font_size = self.style.font_size;
        let font_color = self.style.font_color;
        let tab_count = tabs.len() as u32;

        widget_children::<TabsWidget, _>(|| {
            for title in titles {
                pad(Pad::balanced(12.0, 6.0), || {
                    let mut title = Text::new(font_size, title);
                    title.style.color = font_color;
                    title.show();
                });
            }

            for content in content {
                content();
            }
        }, Self {
            tab_count,
            ..self
        })
    }
}

#[derive(Debug)]
pub struct TabsWidget {
    tab_count: u32,
    selected: u32,

    style: TabsStyle,

    initialized: bool,

    tabs_bar_size: Cell<Vec2>,
    tab_rects: RefCell<Vec<Rect>>,
}

#[derive(Debug)]
pub struct TabsResponse;

impl Widget for TabsWidget {
    type Props<'widget> = Tabs;
    type Response = TabsResponse;

    fn new() -> Self {
        Self {
            tab_count: 0,
            selected: 0,

            style: TabsStyle::default(),

            initialized: false,

            tabs_bar_size: Cell::new(Vec2::ZERO),
            tab_rects: RefCell::new(Vec::new()),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        if !self.initialized {
            self.tab_count = props.tab_count;
            self.selected = props.selected;
            self.style = props.style;

            self.initialized = true;
        }

        self.tab_rects = RefCell::new(Vec::with_capacity(self.tab_count as usize));

        Self::Response {}
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        ctx.layout.enable_clipping(ctx.dom);

        let node = ctx.dom.get_current();

        let mut tabs_size = Vec2::ZERO;
        let mut tab_rects = self.tab_rects.borrow_mut();
        tab_rects.clear();
        for &id in node.children.iter().take(self.tab_count as usize) {
            let size = ctx.calculate_layout(id, Constraints {
                min: Vec2::ZERO,
                max: Vec2::new(f32::INFINITY, f32::INFINITY),
            });

            let mut position = Vec2::new(tabs_size.x, 0.0);
            position.x += self.style.tab_offset * tab_rects.len() as f32;
            ctx.layout.set_pos(id, position);

            tabs_size.x += size.x;
            tabs_size.y = tabs_size.y.max(size.y);

            tab_rects.push(Rect::from_pos_size(
                position,
                size,
            ))
        }
        drop(tab_rects);
        self.tabs_bar_size.set(tabs_size);

        let content_constraints = Constraints {
            min: Vec2::ZERO,
            max: Vec2::new(constraints.max.x, constraints.max.y - tabs_size.y),
        };

        let content_index = self.tab_count + self.selected;
        if let Some(&id) = node.children.get(content_index as usize) {
            ctx.calculate_layout(id, content_constraints);

            ctx.layout.set_pos(id, Vec2::new(0.0, tabs_size.y));
        }

        constraints.max
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let node = ctx.dom.get_current();
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        let position = layout_node.rect.pos();

        let tab_rects = self.tab_rects.borrow();

        for (index, &tab_id) in node.children.iter().take(self.tab_count as usize).enumerate() {
            let tab_rect = tab_rects[index];

            draw_rect(
                &mut ctx,
                position + tab_rect.pos(),
                tab_rect.size(),
                self.style.tab_background,
            );

            ctx.paint(tab_id);
        }
        drop(tab_rects);

        let content_index = (self.tab_count + self.selected) as usize;
        if let Some(&content_id) = node.children.get(content_index) {
            ctx.paint(content_id);
        }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE
    }

    fn event(&mut self, ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match *event {
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down: true,
                inside: true,
                position,
                ..
            } => {
                let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
                let local = position - rect.pos();

                let tab_rects = self.tab_rects.borrow();

                if local.y < self.tabs_bar_size.get().y {
                    for (i, tab_rect) in tab_rects.iter().take(self.tab_count as usize).enumerate() {
                        if tab_rect.contains_point(local) {
                            self.selected = i as u32;

                            return EventResponse::Sink
                        }
                    }
                }

                EventResponse::Bubble
            }
            _ => EventResponse::Bubble,
        }
    }
}

pub fn tabs(theme: &Theme, tabs: &[(&str, &dyn Fn())], selected: u32) -> Response<TabsResponse> {
    Tabs::new(&theme, selected).show(tabs)
}

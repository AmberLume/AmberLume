use yakui::widgets::{ColoredBox, Draggable, Pad};
use yakui::{Color, Constraints, Vec2, column, constrained, expanded, label, offset, pad, row, stack, colored_box, Alignment, align, scroll_vertical};
use amber_lume::ui::ui_context::UiContext;

pub struct WindowContext {
    pub title: String,

    pub position: Vec2,
    pub size: Vec2,
}

pub fn window(context: &UiContext, window_context: &mut WindowContext, children: impl FnOnce()) {
    let size = window_context.size.max(Vec2::new(200.0, 200.0)).min(context.window_size);
    let position = window_context.position.max(Vec2::ZERO).min(context.window_size - size);

    offset(position, || {
        constrained(Constraints::tight(size), || {
            ColoredBox::container(Color::rgba(30, 30, 30, 200)).show_children(|| {
                column(|| {
                    let drag = Draggable::new().show(|| {
                        header(&window_context);
                    });

                    if let Some(delta) = drag.dragging {
                        window_context.position = delta.current;
                    }

                    expanded(|| {
                        stack(|| {
                            scroll_vertical(children);

                            align(Alignment::BOTTOM_RIGHT, || {
                                let grip_size = Vec2::new(16.0, 16.0);

                                let drag = Draggable::new().show(|| {
                                    colored_box(Color::RED, grip_size);
                                });

                                if let Some(delta) = drag.dragging {
                                    window_context.size = delta.current + grip_size - position;
                                }
                            });
                        });
                    });
                });
            });
        });
    });
}

fn header(context: &WindowContext) {
    ColoredBox::container(Color::rgba(128, 128, 128, 255)).show_children(|| {
        pad(Pad::balanced(4.0, 2.0), || {
            row(|| {
                expanded(|| {
                    label(context.title.clone());
                });
            });
        });
    });
}

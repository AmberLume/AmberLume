use yakui::paint::PaintRect;
use yakui::{Color, Rect, Vec2};
use yakui::widget::PaintContext;

pub fn draw_rect(ctx: &mut PaintContext, position: Vec2, size: Vec2, color: Color) {
    let mut border = PaintRect::new(Rect::from_pos_size(position, size));
    border.color = color;

    border.add(ctx.paint);
}
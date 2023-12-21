use crate::math::*;
use crate::*;

pub struct Ui {
    shapes: ShapeBatch,
    sprites: SpriteBatch,
    font: Font,
    ui_matrix_i: Mat4,
}

impl Ui {
    pub fn new(ctx: &Context, font: Font) -> Self {
        Self {
            shapes: ShapeBatch::new(ctx),
            sprites: SpriteBatch::new(ctx),
            font,
            ui_matrix_i: Mat4::default(),
        }
    }

    pub fn update_matrix(&mut self, matrix: Mat4) {
        self.sprites.set_model_view_projection_matrix(matrix);
        self.shapes.set_model_view_projection_matrix(matrix);
        self.ui_matrix_i = matrix.inverse();
    }

    pub fn draw(&mut self, ctx: &Context) {
        self.shapes.draw(ctx);
        self.sprites.draw(ctx);
    }

    pub fn button(&mut self, ctx: &Context, pos: Vec2, text: &str) -> bool {
        let dim = self.font.measure(text);
        self.shapes
            .add_filled_rect(ctx, pos, pos + dim + vec2(6.0, 6.0), BLUE);
        self.font
            .draw_text(ctx, &mut self.sprites, text, pos + vec2(3.0, 3.0), WHITE);
        ctx.is_mouse_button_pressed(MouseButton::Left) && {
            let mp = vec3(
                ctx.mouse_position().x as f32 / ctx.screen_size().x as f32 * 2.0 - 1.0,
                ctx.mouse_position().y as f32 / ctx.screen_size().y as f32 * -2.0 + 1.0,
                0.0,
            );
            let mp = self.ui_matrix_i.transform_point3(mp).truncate();
            mp.x >= pos.x
                && mp.x <= pos.x + dim.x + 6.0
                && mp.y >= pos.y
                && mp.y <= pos.y + dim.y + 6.0
        }
    }
}

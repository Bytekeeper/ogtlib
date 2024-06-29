use crate::math::*;
use crate::*;
use ahash::AHashMap;
use std::borrow::Borrow;
use std::cell::Cell;
use std::sync::Arc;

pub struct Ui {
    shapes: ShapeBatch,
    sprites: SpriteBatch,
    font: Font,
    ui_matrix_i: Mat4,
}

pub trait LayoutElement {
    fn prefered_dimensions(&self, ui: &Ui) -> Vec2;
    fn set_rect(&self, ui: &Ui, xy: Vec2, dim: Vec2);
    fn render(&self, ctx: &Context, ui: &mut Ui, mouse_pos: Vec2);

    fn layout(&self, ui: &Ui, xy: Vec2) {
        let dim = self.prefered_dimensions(ui);
        self.set_rect(ui, xy, dim);
    }
}

pub struct Input<'a> {
    xy: Cell<Vec2>,
    wh: Cell<Option<Vec2>>,
    text: &'a mut str,
}

impl<'a> Input<'a> {
    pub fn new(ctx: &Context, text: &'a mut str) -> Self {
        Self {
            xy: Cell::new(Vec2::ZERO),
            wh: Cell::new(None),
            text,
        }
    }
}

pub struct Label<S> {
    xy: Cell<Vec2>,
    wh: Cell<Option<Vec2>>,
    text: S,
}

impl<S: Borrow<str>> Label<S> {
    pub fn new(text: S) -> Self {
        Self {
            text,
            xy: Cell::new(Vec2::ZERO),
            wh: Cell::new(None),
        }
    }
}

impl<S: Borrow<str>> LayoutElement for Label<S> {
    fn render(&self, ctx: &Context, ui: &mut Ui, mouse_pos: Vec2) {
        let dim = ui.font.measure(self.text.borrow());
        let wh = self.wh.get().unwrap_or_else(|| dim.1 - dim.0);
        let xy = self.xy.get();
        let tr = xy + wh;
        ui.font.draw_text(
            ctx,
            &mut ui.sprites,
            self.text.borrow(),
            xy + wh / 2.0 - (dim.1 + dim.0) / 2.0, // Not dim.1 - dim.0 because we need the center not the dimensions
            WHITE,
        );
    }

    fn prefered_dimensions(&self, ui: &Ui) -> Vec2 {
        self.wh.get().unwrap_or_else(|| {
            let dim = ui.font.measure(self.text.borrow());
            dim.1 - dim.0
        })
    }

    fn set_rect(&self, ui: &Ui, xy: Vec2, dim: Vec2) {
        self.xy.set(xy);
        self.wh.set(Some(dim));
    }
}

pub struct Frame<T> {
    xy: Cell<Vec2>,
    wh: Cell<Vec2>,
    element: T,
}

impl<T> Frame<T> {
    pub fn new(element: T) -> Self {
        Self {
            xy: Cell::new(Vec2::ZERO),
            wh: Cell::new(Vec2::ZERO),
            element,
        }
    }
}

impl<'a, T: LayoutElement> LayoutElement for Frame<&'a T> {
    fn render(&self, ctx: &Context, ui: &mut Ui, mouse_pos: Vec2) {
        let bl = self.xy.get();
        let tr = bl + self.wh.get();
        ui.shapes.add_filled_rect(ctx, bl, tr, LIGHT_GRAY);
        self.element.render(ctx, ui, mouse_pos);
    }

    fn prefered_dimensions(&self, ui: &Ui) -> Vec2 {
        self.element.prefered_dimensions(ui) + vec2(30.0, 30.0)
    }

    fn set_rect(&self, ui: &Ui, xy: Vec2, dim: Vec2) {
        self.xy.set(xy);
        self.wh.set(dim);
        self.element
            .set_rect(ui, xy + vec2(15.0, 15.0), dim - vec2(30.0, 30.0));
    }
}

pub struct Button<S, U = ()> {
    xy: Cell<Vec2>,
    wh: Cell<Option<Vec2>>,
    padding: Vec2,
    text: S,
    pressed: Cell<bool>,
    pub user_data: Option<U>,
}

impl<S: Borrow<str>, U> Button<S, U> {
    pub fn new(text: S) -> Self {
        Self {
            xy: Cell::new(Vec2::ZERO),
            wh: Cell::new(None),
            text: text,
            padding: vec2(20.0, 20.0),
            pressed: Cell::new(false),
            user_data: None,
        }
    }

    pub fn at(self, xy: Vec2) -> Self {
        Self {
            xy: Cell::new(xy),
            ..self
        }
    }

    pub fn user_data(mut self, user_data: U) -> Self {
        self.user_data = Some(user_data);
        self
    }

    pub fn clicked(self) -> Option<U> {
        self.pressed.get().then(|| self.user_data).flatten()
    }
}

impl<S: Borrow<str>, U> LayoutElement for Button<S, U> {
    fn prefered_dimensions(&self, ui: &Ui) -> Vec2 {
        self.wh.get().unwrap_or_else(|| {
            let dim = ui.font.measure(self.text.borrow());
            dim.1 - dim.0 + self.padding
        })
    }

    fn set_rect(&self, ui: &Ui, xy: Vec2, dim: Vec2) {
        self.xy.set(xy);
        self.wh.set(Some(dim));
    }

    fn render(&self, ctx: &Context, ui: &mut Ui, mp: Vec2) {
        let dim = ui.font.measure(self.text.borrow());
        let wh = self
            .wh
            .get()
            .unwrap_or_else(|| dim.1 - dim.0 + self.padding);
        let xy = self.xy.get();
        let tr = xy + wh;
        let mouse_on_button = mp.x >= xy.x && mp.x <= tr.x && mp.y >= xy.y && mp.y <= tr.y;
        ui.shapes.add_filled_rect(
            ctx,
            xy,
            tr,
            if mouse_on_button && !ctx.is_mouse_button_down(MouseButton::Left) {
                LIGHT_BLUE
            } else {
                BLUE
            },
        );
        ui.font.draw_text(
            ctx,
            &mut ui.sprites,
            self.text.borrow(),
            xy + wh / 2.0 - (dim.1 + dim.0) / 2.0, // Not dim.1 - dim.0 because we need the center not the dimensions
            WHITE,
        );
        self.pressed
            .set(ctx.is_mouse_button_pressed(MouseButton::Left) && mouse_on_button);
    }
}

pub struct VerticalLayout<T> {
    elements: T,
    gap: f32,
}

impl<'a, T: LayoutElement> LayoutElement for VerticalLayout<&[T]> {
    fn prefered_dimensions(&self, ui: &Ui) -> Vec2 {
        let mut dim = Vec2::ZERO;
        let mut element_count: usize = 0;
        for element in self.elements.as_ref() {
            let element_dim = element.prefered_dimensions(ui);
            dim.x = dim.x.max(element_dim.x);
            dim.y += element_dim.y;
            element_count += 1;
        }
        dim + element_count.saturating_sub(1) as f32 * vec2(0.0, self.gap)
    }

    fn set_rect(&self, ui: &Ui, mut xy: Vec2, dim: Vec2) {
        for element in self.elements.as_ref() {
            let element_dim = element.prefered_dimensions(ui);
            element.set_rect(ui, xy, vec2(dim.x, element_dim.y));
            xy.y += element_dim.y + self.gap;
        }
    }

    fn render(&self, ctx: &Context, ui: &mut Ui, mouse_pos: Vec2) {
        for element in self.elements.as_ref() {
            element.render(ctx, ui, mouse_pos);
        }
    }
}

impl<T> VerticalLayout<T> {
    pub fn new(elements: T) -> Self {
        Self { elements, gap: 0.0 }
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
}

pub struct CenterLayout<T> {
    element: T,
}

impl<'a, T: LayoutElement> LayoutElement for CenterLayout<T> {
    fn prefered_dimensions(&self, ui: &Ui) -> Vec2 {
        self.element.prefered_dimensions(ui)
    }

    fn set_rect(&self, ui: &Ui, xy: Vec2, dim: Vec2) {
        let child_dim = self.element.prefered_dimensions(ui);
        let center = xy + dim / 2.0;
        self.element
            .set_rect(ui, center - child_dim / 2.0, child_dim);
    }

    fn render(&self, ctx: &Context, ui: &mut Ui, mouse_pos: Vec2) {
        self.element.render(ctx, ui, mouse_pos);
    }
}

impl<T> CenterLayout<T> {
    pub fn new(element: T) -> Self {
        Self { element }
    }
}

#[derive(Default)]
pub struct CornerLayout<'a> {
    top_right: Option<&'a dyn LayoutElement>,
    bottom_right: Option<&'a dyn LayoutElement>,
}

impl<'a> CornerLayout<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn bottom_right(mut self, top_right: &'a dyn LayoutElement) -> Self {
        self.bottom_right = Some(top_right);
        self
    }

    pub fn top_right(mut self, top_right: &'a dyn LayoutElement) -> Self {
        self.top_right = Some(top_right);
        self
    }
}

impl<'a> LayoutElement for CornerLayout<'a> {
    fn prefered_dimensions(&self, ui: &Ui) -> Vec2 {
        let mut dim = Vec2::ZERO;
        for element in [self.top_right, self.bottom_right].iter().flatten() {
            dim += element.prefered_dimensions(ui);
        }
        dim
    }

    fn set_rect(&self, ui: &Ui, xy: Vec2, dim: Vec2) {
        self.top_right.map(|br| {
            let dim_top_right = br.prefered_dimensions(ui);
            br.set_rect(ui, xy + dim - dim_top_right, dim_top_right)
        });
        self.bottom_right.map(|br| {
            let dim_top_right = br.prefered_dimensions(ui);
            br.set_rect(
                ui,
                xy + (dim - dim_top_right) * vec2(1.0, 0.0),
                dim_top_right,
            )
        });
    }

    fn render(&self, ctx: &Context, ui: &mut Ui, mouse_pos: Vec2) {
        self.top_right.map(|tr| tr.render(ctx, ui, mouse_pos));
        self.bottom_right.map(|br| br.render(ctx, ui, mouse_pos));
    }
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

    pub fn screen_to_ui(&self, point: Vec2) -> Vec2 {
        self.ui_matrix_i
            .project_point3(point.extend(0.0))
            .truncate()
    }

    pub fn update_matrix(&mut self, matrix: Mat4) {
        self.sprites.set_model_view_projection_matrix(matrix);
        self.shapes.set_model_view_projection_matrix(matrix);
        self.ui_matrix_i = matrix.inverse();
    }

    pub fn render(&mut self, ctx: &Context, item: &impl LayoutElement) {
        let mouse_pos = vec3(
            ctx.mouse_position().x as f32 / ctx.screen_size().x as f32 * 2.0 - 1.0,
            ctx.mouse_position().y as f32 / ctx.screen_size().y as f32 * -2.0 + 1.0,
            0.0,
        );
        let mouse_pos = self.ui_matrix_i.transform_point3(mouse_pos).truncate();
        item.render(ctx, self, mouse_pos);
    }

    pub fn draw(&mut self, ctx: &Context) {
        self.shapes.draw(ctx);
        self.sprites.draw(ctx);
    }
}

use crate::backend::load_file;
use crate::{Context, Font, LoadedFont, Texture, TextureBuilder};
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

pub struct Ref<'a, T>(RefMut<'a, T>);

impl<T> std::ops::Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

enum Inner<T, D> {
    Empty,
    Loaded(D),
    Ready(T),
}

impl<T> Inner<T, Vec<u8>> {
    fn try_readying(&mut self, build: impl FnOnce(&Vec<u8>) -> T) {
        match self {
            Inner::Empty | Inner::Ready(_) => (),
            Inner::Loaded(data) => {
                let built = build(data);
                *self = Inner::Ready(built);
            }
        }
    }
}

#[derive(Clone)]
pub struct Asset<T, D = Vec<u8>> {
    inner: Rc<RefCell<Inner<T, D>>>,
}

impl<T: 'static> Asset<T> {
    pub fn load(file: &str) -> Asset<T> {
        let inner = Rc::new(RefCell::new(Inner::Empty));
        let loader = inner.clone();
        load_file(file, move |result| match result {
            Ok(content) => {
                *loader.borrow_mut() = Inner::Loaded(content);
            }
            Err(err) => eprintln!("Could not load {:?}", err),
        });
        Asset { inner }
    }

    fn get_or_build(&self, build: impl FnOnce(&Vec<u8>) -> T) -> Option<Ref<T>> {
        let mut inner = self.inner.borrow_mut();
        inner.try_readying(build);
        match *inner {
            Inner::Empty => None,
            Inner::Loaded(_) => unreachable!(),
            Inner::Ready(_) => Some(Ref(RefMut::map(inner, |item| match item {
                Inner::Ready(item) => item,
                _ => unreachable!(),
            }))),
        }
    }
}

impl<T, D> Asset<T, D> {
    pub fn take(&self) -> Option<T> {
        let mut inner = self.inner.borrow_mut();
        let inner = std::mem::replace(&mut *inner, Inner::Empty);
        match inner {
            Inner::Empty | Inner::Loaded(_) => None,
            Inner::Ready(result) => Some(result),
        }
    }
}

impl Asset<Texture> {
    pub fn get(&self, ctx: &Context) -> Option<Ref<Texture>> {
        self.get_or_build(|content| {
            let image = image::load_from_memory(&content).unwrap().to_rgba8();
            TextureBuilder::from_bytes(image.as_raw(), image.width(), image.height()).build(ctx)
        })
    }
}

impl Asset<LoadedFont> {
    pub fn get(&self, _ctx: &Context) -> Option<Ref<LoadedFont>> {
        self.get_or_build(|content| LoadedFont::from_bytes(content))
    }
}

pub struct FontSource {
    pub size: f32,
    pub source: Asset<LoadedFont>,
}

impl Asset<Font, FontSource> {
    pub fn new(source: FontSource) -> Self {
        let inner = Rc::new(RefCell::new(Inner::Loaded(source)));
        Self { inner }
    }

    pub fn get(&self, ctx: &Context) -> Option<Ref<Font>> {
        let mut inner = self.inner.borrow_mut();
        match *inner {
            Inner::Empty => {
                return None;
            }
            Inner::Loaded(FontSource { ref source, size }) => {
                if let Some(font) = source.get(ctx).map(|fnt| fnt.create_font(ctx, size)) {
                    *inner = Inner::Ready(font);
                } else {
                    return None;
                }
            }
            Inner::Ready(_) => {}
        }
        Some(Ref(RefMut::map(inner, |item| match item {
            Inner::Ready(item) => item,
            _ => unreachable!(),
        })))
    }
}

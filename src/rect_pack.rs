use std::cell::Cell;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn wh(width: u32, height: u32) -> Rect {
        Rect {
            x: 0,
            y: 0,
            width,
            height,
        }
    }
}

pub fn pack(rects: &mut [Rect], max_width: u32) -> Option<(u32, u32)> {
    let mut indices: Box<_> = (0..rects.len()).collect();
    indices.sort_by_key(|&i| std::cmp::Reverse(rects[i].height));
    let mut max_w = max_width;
    let mut best: Option<(u32, u32, Vec<Rect>)> = None;
    while let Some((w, h)) = {
        let rects = Cell::from_mut(rects).as_slice_of_cells();
        pack_limited(&mut indices.iter().map(|&i| &rects[i]), max_w)
    } {
        if let Some(ref b) = best {
            if b.0 * b.1 > w * h {
                best = Some((w, h, rects.to_vec()));
            }
        } else {
            best = Some((w, h, rects.to_vec()));
        }
        max_w = w - 1;
    }
    if let Some(best) = best {
        rects.copy_from_slice(&best.2[..]);
        Some((best.0, best.1))
    } else {
        None
    }
}

fn pack_limited(
    rects: &mut dyn Iterator<Item = &Cell<Rect>>,
    max_width: u32,
) -> Option<(u32, u32)> {
    let mut spaces = vec![Rect {
        x: 0,
        y: 0,
        width: max_width,
        height: 16384,
    }];

    let mut w = 0;
    let mut h = 0;
    for rect in rects {
        let mut update = rect.get();
        if let Some((space_index, &space)) = spaces
            .iter()
            .enumerate()
            .rev()
            .filter(|(i, s)| s.width >= update.width && s.height >= update.height)
            .next()
        {
            update.x = space.x;
            update.y = space.y;
            rect.set(update);
            let rect = update;
            w = w.max(rect.x + rect.width);
            h = h.max(rect.y + rect.height);
            spaces.remove(space_index);
            let free_w = space.width - rect.width;
            let free_h = space.height - rect.height;

            if free_w > 0 && free_w >= free_h {
                let bigger = Rect {
                    x: space.x + rect.width,
                    y: space.y,
                    width: free_w,
                    height: space.height,
                };
                spaces.push(bigger);
                if free_h > 0 {
                    let lesser = Rect {
                        x: space.x,
                        y: space.y + rect.height,
                        width: rect.width,
                        height: free_h,
                    };
                    spaces.push(lesser);
                }
            } else if free_h > free_w {
                let bigger = Rect {
                    x: space.x,
                    y: space.y + rect.height,
                    width: space.width,
                    height: free_h,
                };
                spaces.push(bigger);
                if free_w > 0 {
                    let lesser = Rect {
                        x: space.x + rect.width,
                        y: space.y,
                        width: free_w,
                        height: rect.height,
                    };
                    spaces.push(lesser);
                }
            }
        } else {
            return None;
        }
    }
    Some((w, h))
}

#[cfg(test)]
mod test {
    use super::*;
    use image::*;
    use imageproc::drawing::*;
    use imageproc::rect::Rect as r;
    use quad_rand::*;

    #[test]
    fn pack_random_200() {
        let mut rects = vec![];
        srand(0);
        for _ in 0..200 {
            rects.push(Rect::wh(gen_range(7, 50), gen_range(7, 50)));
        }
        let dim = pack(&mut rects, 4096);

        assert!(dim.is_some(), "No solution was found");
        assert!(
            rects.iter().all(|a| rects.iter().all(|b| a == b
                || a.x >= b.x + b.width
                || a.y >= b.y + b.height
                || a.x + a.width <= b.x
                || a.y + a.height <= b.y)),
            "Overlapping regions found"
        );

        #[cfg(feature = "debug_images")]
        {
            let mut image = RgbImage::new(dim.0, dim.1);
            for rect in rects.iter() {
                draw_filled_rect_mut(
                    &mut image,
                    r::at(rect.x as i32, rect.y as i32).of_size(rect.width, rect.height),
                    Rgb([gen_range(30, 255), gen_range(30, 255), gen_range(30, 255)]),
                );
            }
            save_buffer(
                "/tmp/packed.png",
                &image.into_raw(),
                dim.0,
                dim.1,
                ColorType::Rgb8,
            );
        }
    }
}

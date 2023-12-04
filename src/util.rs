use bevy::math::Vec2;

pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

struct Intercept {
    pos: f32,
    edge: Edge,
}

/// Given a direction vector and the minimum and maximum bounds of a rectangle,
/// project a ray from 0, 0 and return the intersection with that rectangle.
/// https://math.stackexchange.com/questions/2738250/intersection-of-ray-starting-inside-square-with-that-square
pub fn project_onto_bounding_rectangle(dir: Vec2, min: Vec2, max: Vec2) -> Option<(Vec2, Edge)> {
    let maybe_x_int: Option<Intercept> = if dir.x > 0. {
        Some(Intercept {
            pos: max.x / dir.x,
            edge: Edge::Right,
        })
    } else if dir.x < 0. {
        Some(Intercept {
            pos: min.x / dir.x,
            edge: Edge::Left,
        })
    } else {
        None
    };

    let maybe_y_int: Option<Intercept> = if dir.y > 0. {
        Some(Intercept {
            pos: max.y / dir.y,
            edge: Edge::Top,
        })
    } else if dir.y < 0. {
        Some(Intercept {
            pos: min.y / dir.y,
            edge: Edge::Bottom,
        })
    } else {
        None
    };

    match (maybe_x_int, maybe_y_int) {
        (None, Some(y_int)) => Some((Vec2::new(y_int.pos * dir.x, y_int.pos * dir.y), y_int.edge)),
        (Some(x_int), None) => Some((Vec2::new(x_int.pos * dir.x, x_int.pos * dir.y), x_int.edge)),
        (Some(x_int), Some(y_int)) => {
            if x_int.pos < y_int.pos {
                Some((Vec2::new(x_int.pos * dir.x, x_int.pos * dir.y), x_int.edge))
            } else if y_int.pos < x_int.pos {
                Some((Vec2::new(y_int.pos * dir.x, y_int.pos * dir.y), y_int.edge))
            } else {
                Some((Vec2::new(x_int.pos * dir.x, x_int.pos * dir.y), x_int.edge))
            }
        }
        (None, None) => None,
    }
}

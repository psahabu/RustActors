use std::f64;

struct Point {
	x: f64,
	y: f64
}

enum Shape {
	Circle(Point, f64),
	Rectangle(Point, Point)
}

fn main() {
	let origin = Point {x: 0.0, y: 0.0};
	let incr = incr_y(incr_x(origin));
	println!("{}", area(Rectangle(origin, incr)));
}

fn incr_x(p: Point) -> Point {
	modify_x(p, |x| x+1.0)
}

fn incr_y(p: Point) -> Point {
	modify_y(p, |y| y+1.0)
}

fn modify_x(Point {x: x, y: y}: Point, modify: |f64| -> f64) -> Point {
	Point {x: modify(x), y: y}
}

fn modify_y(Point {x: x, y: y}: Point, modify: |f64| -> f64) -> Point {
	Point {x: x, y: modify(y)}
}

fn area(s: Shape) -> f64 {
	match s {
		Circle(_, r) => f64::consts::PI * r * r ,
		Rectangle(bot_l, top_r) => (top_r.x-bot_l.x) * (top_r.y-bot_l.y)
	}
}

/*
macro_rules! modify_coord(
	($mod_dim:ident $same_dim:ident) => (
		fn modify_+$mod_dim(Point {x: x, y: y}: Point, modify: |f64| -> f64) -> Point {
			Point {$mod_dim: modify($mod_dim), $same_dim: $same_dim}
		}
	);
)
*/

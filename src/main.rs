use nannou::{prelude::*, rand::random_f32};

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    app.new_window()
        .view(view)
        .mouse_pressed(mouse_pressed)
        .mouse_moved(mouse_moved)
        .mouse_released(mouse_released)
        .build()
        .unwrap();

    let start = Point2::new(0.0, 0.0);
    let end = Point2::new(100.0, 0.0);
    let count = 12;

    Model {
        balls: vec![],
        rope: Rope::new(start, end, count),
        enemies: vec![],
        is_dragging: false,
        drag_index: Some(0),
        enemy_timer: 0.0,
        spawn_delay: 0.5,
        camera_position: vec2(0.0, 0.0),
    }
}

struct Rope {
    points: Vec<Point2>,
    prev_points: Vec<Point2>,
    segment_length: f32,
    thickness: f32,
    color: Rgba,
}

impl Rope {
    fn new(start: Point2, end: Point2, count: usize) -> Self {
        let length = start.distance(end);
        let segment_length = length / (count as f32 - 1.0);
        let direction = (end - start).normalize();

        let points: Vec<Point2> = (0..count)
            .map(|i| start + direction * segment_length * i as f32)
            .collect();

        let prev_points = points.clone();

        Rope {
            points,
            prev_points,
            segment_length,
            thickness: 4.0,
            color: nannou::color::Rgba::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    fn update(&mut self) {
        self.update_rope();
    }

    fn update_rope(&mut self) {
        let gravity = vec2(0.0, -0.1);

        for i in 1..self.points.len() {
            let current = self.points[i];
            let prev = self.prev_points[i];
            let velocity = current - prev;
            let next_position = current + velocity; // + gravity;
            self.prev_points[i] = self.points[i];
            self.points[i] = next_position;
        }

        self.constrain_points();
    }

    fn constrain_points(&mut self) {
        let count = self.points.len();
        for _ in 0..5 {
            for i in 0..(count - 1) {
                let point_a = self.points[i];
                let point_b = self.points[i + 1];
                let delta = point_b - point_a;
                let distance = delta.length();
                let difference = self.segment_length - distance;
                let correction = delta.normalize() * (difference / 1.2);
                if i != 0 {
                    self.points[i] -= correction;
                }
                self.points[i + 1] += correction;
            }
        }
    }
}

struct Enemy {
    position: Point2,
    radius: f32,
    color: Rgba,
}

impl Enemy {
    fn new(position: Point2, radius: f32, color: Rgba) -> Self {
        Enemy {
            position,
            radius,
            color,
        }
    }
}

struct Ball {
    position: Point2,
    velocity: Vector2,
    radius: f32,
    color: Rgba,
}

struct Model {
    balls: Vec<Ball>,
    enemies: Vec<Enemy>,
    rope: Rope,
    is_dragging: bool,
    drag_index: Option<usize>,
    enemy_timer: f32,
    spawn_delay: f32,
    camera_position: Vector2,
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.enemy_timer += 0.01;
    model.rope.update();
    if model.is_dragging {
        if let Some(index) = model.drag_index {
            let cursor_position = _app.mouse.position();
            let current_position = model.rope.points[index];
            let lerp_position = lerp(current_position, cursor_position, 0.3);
            model.rope.points[index] = lerp_position;
        }
    }
    spawn_enemies(_app, model);

    // Lerp camera position to the first point of the rope
    let target_position = model.rope.points[0];
    model.camera_position = lerp_vec2(model.camera_position, target_position as Vec2, 0.1);
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.is_dragging = true;
    model.drag_index = Some(0); // Drag the first point
}

fn mouse_moved(_app: &App, model: &mut Model, _position: Point2) {
    if model.is_dragging {
        if let Some(index) = model.drag_index {
            model.rope.points[index] = lerp(model.rope.points[index], _position, 0.1);
        }
    }
}

fn mouse_released(_app: &App, model: &mut Model, _button: MouseButton) {
    model.is_dragging = false;
    model.drag_index = None;
}

fn view(app: &App, model: &Model, frame: Frame) {
    // Begin drawing
    let draw = app.draw();

    // Clear the background to black.
    draw.background().color(BLACK);

    // Apply camera transformation
    draw.x_y(-model.camera_position.x, -model.camera_position.y);

    for (i, point) in model.rope.points.iter().enumerate() {
        let radius = if i == 0 || i == model.rope.points.len() - 1 {
            model.rope.thickness * 2.0 // First and last points are larger
        } else {
            model.rope.thickness / 2.0
        };

        draw.ellipse()
            .x_y(point.x, point.y)
            .radius(radius)
            .color(model.rope.color);
    }
    for enemy in model.enemies.iter() {
        draw.ellipse()
            .x_y(enemy.position.x, enemy.position.y)
            .radius(enemy.radius)
            .color(enemy.color);
    }

    // Write the result of our drawing to the window's frame.
    draw.to_frame(app, &frame).unwrap();
}

fn lerp(a: Point2, b: Point2, t: f32) -> Point2 {
    let x = a.x + (b.x - a.x) * t;
    let y = a.y + (b.y - a.y) * t;
    Point2::new(x, y)
}

fn lerp_vec2(a: Vector2, b: Vector2, t: f32) -> Vector2 {
    let x = a.x + (b.x - a.x) * t;
    let y = a.y + (b.y - a.y) * t;
    vec2(x, y)
}

fn spawn_enemies(app: &App, model: &mut Model) {
    if model.enemy_timer >= model.spawn_delay {
        let win = app.window_rect();
        let x = random_f32() * win.w() - win.w() / 2.0;
        let y = random_f32() * win.h() - win.h() / 2.0;
        let position = Point2::new(x, y);
        let radius = random_range(5.0, 20.0);
        let color = Rgba::new(random_f32(), random_f32(), random_f32(), 1.0);
        model.enemies.push(Enemy::new(position, radius, color));
        model.enemy_timer = 0.0;
    }
}

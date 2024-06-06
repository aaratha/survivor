use nannou::{prelude::*, rand::random_f32};
use text::glyph::X;

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    app.new_window()
        .view(view)
        .mouse_pressed(mouse_pressed)
        .mouse_released(mouse_released)
        .build()
        .unwrap();

    let start = Point2::new(0.0, 0.0);
    let end = Point2::new(100.0, 0.0);
    let count = 12;

    Model {
        rope: Rope::new(start, end, count),
        enemies: vec![],
        is_dragging: false,
        drag_index: Some(0),
        enemy_timer: 0.0,
        spawn_delay: 0.5,
        camera_position: vec2(0.0, 0.0),
        score: 0,
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

    fn update(&mut self, substeps: i32) {
        self.update_rope(substeps);
    }

    fn update_rope(&mut self, substeps: i32) {
        let dt = 1.0 / substeps as f32;

        for i in 1..self.points.len() {
            let current = self.points[i];
            let prev = self.prev_points[i];
            let velocity = current - prev;
            let next_position = current + velocity; // Apply gravity here if needed
            self.prev_points[i] = self.points[i];
            self.points[i] = next_position;
        }

        for _ in 0..substeps {
            self.constrain_points();
        }
    }

    fn constrain_points(&mut self) {
        let count = self.points.len();
        for _ in 0..3 {
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

    fn get_segment_midpoints(&self) -> Vec<Point2> {
        let mut midpoints = vec![];
        for i in 0..(self.points.len() - 1) {
            let midpoint = (self.points[i] + self.points[i + 1]) * 0.5;
            midpoints.push(midpoint);
        }
        midpoints
    }
}

struct Enemy {
    position: Point2,
    prev_position: Point2,
    radius: f32,
    color: Rgba,
}

impl Enemy {
    fn new(position: Point2, radius: f32, color: Rgba) -> Self {
        Enemy {
            position,
            prev_position: position,
            radius,
            color,
        }
    }

    fn update(&mut self, target: Point2, delta_time: f32) {
        let current = self.position;
        let prev = self.prev_position;
        let velocity = current - prev;
        self.prev_position = current;

        // Move towards the target (first point of the rope)
        let direction = (target - current).normalize();
        let next_position = current + velocity + direction * delta_time;
        self.position = next_position;
    }
}

struct Model {
    enemies: Vec<Enemy>,
    rope: Rope,
    is_dragging: bool,
    drag_index: Option<usize>,
    enemy_timer: f32,
    spawn_delay: f32,
    camera_position: Vector2,
    score: i32,
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    model.enemy_timer += 0.01;
    let substeps = 5; // Number of substeps for more accurate updates
    let delta_time = 0.01 / substeps as f32;

    let target_position = model.rope.points[0];
    for _ in 0..substeps {
        model.rope.update(substeps);
        if model.is_dragging {
            if let Some(index) = model.drag_index {
                let cursor_position = _app.mouse.position();
                let current_position = model.rope.points[index];
                let lerp_position = lerp(current_position, cursor_position, 0.06);
                model.rope.points[index] = lerp_position;
            }
        }

        // Update enemies to move towards the first rope point
        for enemy in model.enemies.iter_mut() {
            enemy.update(target_position, delta_time);
        }

        // Check for collisions
        check_collisions(&mut model.rope, &mut model.enemies, substeps);
    }

    spawn_enemies(_app, model);
    despawn_enemies(_app, model);

    // Lerp camera position to the first point of the rope
    model.camera_position = lerp_vec2(model.camera_position, target_position as Vec2, 0.1);
}

fn check_collisions(rope: &mut Rope, enemies: &mut [Enemy], substeps: i32) {
    let midpoints = rope.get_segment_midpoints();

    for enemy in enemies.iter_mut() {
        for point in rope.points.iter_mut() {
            let distance = enemy.position.distance(*point + vec2(rope.thickness, 0.0));
            if distance < enemy.radius {
                // Simple collision response: move both enemy and rope point away from each other
                let direction = (enemy.position - *point).normalize();
                let overlap = (enemy.radius - distance) / substeps as f32;
                enemy.position += direction * overlap * 0.5;
                *point -= direction * overlap * 0.5;
            }
        }

        for midpoint in midpoints.iter() {
            let distance = enemy.position.distance(*midpoint);
            let dynamic_thickness = rope.segment_length / 2.0;
            if distance < enemy.radius + dynamic_thickness {
                let direction = (enemy.position - *midpoint).normalize();
                let overlap = (enemy.radius + dynamic_thickness - distance) / substeps as f32;
                enemy.position += direction * overlap * 0.5;
            }
        }
    }

    for i in 0..enemies.len() {
        for j in i + 1..enemies.len() {
            let distance = enemies[i].position.distance(enemies[j].position);
            if distance < enemies[i].radius + enemies[j].radius {
                // Simple collision response: move both enemies away from each other
                let direction = (enemies[i].position - enemies[j].position).normalize();
                let overlap = (enemies[i].radius + enemies[j].radius - distance) / substeps as f32;
                enemies[i].position += direction * overlap * 0.5;
                enemies[j].position -= direction * overlap * 0.5;
            }
        }
    }
}

fn mouse_pressed(_app: &App, model: &mut Model, _button: MouseButton) {
    model.is_dragging = true;
    model.drag_index = Some(0); // Drag the first point
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
            model.rope.thickness
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

    draw.text(&model.score.to_string())
        .x_y(
            -app.window_rect().right() + 50.0,
            app.window_rect().top() - 50.0,
        )
        .color(WHITE)
        .font_size(48);

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
        let margin = 1.0; // Margin outside the window
        let (x, y) = if random_f32() < 0.5 {
            // Spawn on the left or right edge
            let x = if random_f32() < 0.5 {
                win.left() - margin
            } else {
                win.right() + margin
            };
            let y = random_f32() * win.h();
            (x, y)
        } else {
            // Spawn on the top or bottom edge
            let x = random_f32() * win.w();
            let y = if random_f32() < 0.5 {
                win.bottom() - margin
            } else {
                win.top() + margin
            };
            (x, y)
        };
        let position = Point2::new(x, y);
        let radius = random_range(10.0, 20.0);
        let color = Rgba::new(random_f32(), random_f32(), random_f32(), 1.0);
        model.enemies.push(Enemy::new(position, radius, color));
        model.enemy_timer = 0.0;
    }
}

fn despawn_enemies(app: &App, model: &mut Model) {
    let win = app.window_rect();
    let margin = 500.0; // Twice the margin used in spawn_enemies
    let mut i = 0;
    while i < model.enemies.len() {
        let x = model.enemies[i].position.x;
        let y = model.enemies[i].position.y;
        let radius = model.enemies[i].radius;
        if x + radius < win.left() - margin
            || x - radius > win.right() + margin
            || y + radius < win.bottom() - margin
            || y - radius > win.top() + margin
        {
            model.enemies.remove(i);
            model.score += 1; // Increase the score
        } else {
            i += 1;
        }
    }
}

use std::process;

use nannou::{
    prelude::*,
    rand::{random_f32, random_range},
};

const SUBSTEPS: usize = 5; // Number of substeps for each update

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
    let end = Point2::new(40.0, 0.0);
    let count = 4;

    Model {
        rope: Rope::new(start, end, count),
        enemies: vec![],
        is_dragging: false,
        drag_index: Some(0),
        enemy_timer: 0.0,
        spawn_delay: 1.0,
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
            segment_length, // Update segment length here
            thickness: 4.0,
            color: nannou::color::Rgba::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    fn update(&mut self) {
        // Perform Verlet integration with sub-stepping
        let substep_delta = 1.0 / SUBSTEPS as f32;
        for _ in 0..SUBSTEPS {
            self.verlet_integration(substep_delta);
            self.constrain_points();
        }
    }

    fn verlet_integration(&mut self, dt: f32) {
        for i in 1..self.points.len() - 1 {
            let velocity = self.points[i] - self.prev_points[i];
            let next_position = self.points[i] + velocity * dt * dt; // Add gravity
            self.prev_points[i] = self.points[i];
            self.points[i] = next_position;
        }
    }

    fn constrain_points(&mut self) {
        let count = self.points.len();
        let segment_length = self.segment_length;
        for _ in 0..5 {
            for i in 0..(count - 1) {
                let point_a = self.points[i];
                let point_b = self.points[i + 1];
                let delta = point_b - point_a;
                let distance = delta.length();
                let difference = segment_length - distance;
                let correction = delta.normalize() * (difference / 1.1);
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

    fn update(&mut self, target: Point2) {
        let current = self.position;
        let prev = self.prev_position;
        let velocity = current - prev;
        self.prev_position = current;

        // Move towards the target (first point of the rope)
        let direction = (target - current).normalize();
        let next_position = current + velocity + direction * 0.1;
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
    for _ in 0..SUBSTEPS {
        model.rope.update();
    }
    if model.is_dragging {
        if let Some(index) = model.drag_index {
            let cursor_position = _app.mouse.position();
            let current_position = model.rope.points[index];
            let lerp_position = lerp(current_position, cursor_position, 0.3);
            model.rope.points[index] = lerp_position;
        }
    }
    spawn_enemies(_app, model);
    despawn_enemies(_app, &mut model.enemies);

    // Update enemies to move towards the first rope point
    let target_position = model.rope.points[0];
    for enemy in model.enemies.iter_mut() {
        enemy.update(target_position);
    }

    // Check for collisions
    check_collisions(&mut model.rope, &mut model.enemies);

    // Lerp camera position to the first point of the rope
    model.camera_position = lerp_vec2(model.camera_position, target_position.into(), 0.1);

    model.spawn_delay -= 0.0001;

    let game_over = check_collisions(&mut model.rope, &mut model.enemies);

    // Set game over flag if collision detected
    if game_over {
        println!("Game Over!");
        println!("Score: {}", model.score);
        process::exit(0);

        // Additional game over logic can be added here
    }
}

fn check_collisions(rope: &mut Rope, enemies: &mut [Enemy]) -> bool {
    for enemy in enemies.iter_mut() {
        for point in rope.points.iter_mut() {
            let distance = enemy.position.distance(*point + (rope.thickness));
            if distance < enemy.radius {
                // Set game over flag if collision detected
                return true;
            }
        }
    }
    false
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

    // Draw the score
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
        let margin = 50.0; // Margin outside the window
        let mut win = app.window_rect().pad(margin);
        let x = random_f32() * (win.w() - margin * 2.0) + win.left();
        let y = random_f32() * (win.h() - margin * 2.0) + win.bottom();
        let position = Point2::new(x, y);
        let radius = random_range(10.0, 20.0);
        let color = Rgba::new(random_f32(), random_f32(), random_f32(), 1.0);
        model.enemies.push(Enemy::new(position, radius, color));

        model.enemy_timer = 0.0;
        model.score += 1;

        // Update rope length and point count when the score increases
        if model.score % 3 == 0 {
            // Add a new point every 3 score points
            let count = model.rope.points.len() + 1; // Increment point count

            // Calculate the new segment length based on the updated count of points
            let start = model.rope.points[0];
            let end = model.rope.points[model.rope.points.len() - 1];
            let direction = (end - start).normalize();
            let length = start.distance(end);
            let segment_length = length / (count as f32 - 1.0);

            // Update the end point of the rope to maintain the curvature
            let new_end = end + direction * segment_length;
            model.rope = Rope::new(start, new_end, count);
        }
    }
}

fn despawn_enemies(app: &App, enemies: &mut Vec<Enemy>) {
    let margin = 50.0; // Change this to the size of the margin you want
    let mut win = app.window_rect();
    win.pad(margin);
    enemies.retain(|enemy| win.contains(enemy.position));
}

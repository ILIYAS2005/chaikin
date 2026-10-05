use macroquad::prelude::*;

// Структура для представления 2D-точки на экране
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32, // Координата X
    pub y: f32, // Координата Y
}

impl Point {
    // Конструктор для удобного создания нового объекта точки
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Выполняет один шаг алгоритма Чайкина (срезание углов).
/// Принимает список опорных точек и возвращает новый сглаженный массив точек.
pub fn chaikin_step(points: &[Point]) -> Vec<Point> {
    // Для сглаживания углов требуется минимум 3 точки
    if points.len() < 3 {
        return points.to_vec();
    }

    // Выделяем память под новый массив точек (из каждой пары точек получаем 2 новые)
    let mut next_points = Vec::with_capacity(2 * points.len() - 2);

    // Проходим по всем смежным отрезкам между опорными точками
    for i in 0..points.len() - 1 {
        let p0 = points[i];
        let p1 = points[i + 1];

        // Точка Q: находится на 25% расстояния от p0 к p1 (соотношение 3/4 и 1/4)
        let q = Point::new(
            0.75 * p0.x + 0.25 * p1.x,
            0.75 * p0.y + 0.25 * p1.y,
        );
        // Точка R: находится на 75% расстояния от p0 к p1 (соотношение 1/4 и 3/4)
        let r = Point::new(
            0.25 * p0.x + 0.75 * p1.x,
            0.25 * p0.y + 0.75 * p1.y,
        );

        next_points.push(q);
        next_points.push(r);
    }

    next_points
}

/// Выполняет N последовательных шагов итерации алгоритма Чайкина
pub fn chaikin_n_steps(points: &[Point], steps: usize) -> Vec<Point> {
    let mut current = points.to_vec();
    for _ in 0..steps {
        if current.len() < 3 {
            break;
        }
        current = chaikin_step(&current);
    }
    current
}

// Конфигурация окна приложения Macroquad
fn window_conf() -> Conf {
    Conf {
        window_title: "Chaikin Algorithm Animation".to_string(),
        window_width: 1024,
        window_height: 768,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Вектор для хранения опорных точек, поставленных пользователем
    let mut control_points: Vec<Point> = Vec::new();
    // Флаг запуска анимации
    let mut is_animating = false;
    // Текущий шаг алгоритма Чайкина в анимации (от 0 до 7)
    let mut current_step: usize = 0;
    // Таймер для отслеживания интервалов между шагами
    let mut step_timer = 0.0;
    // Длительность отображения одного шага (в секундах)
    let step_duration = 0.5;

    // Индекс перетаскиваемой в данный момент точки (если есть)
    let mut dragged_point_index: Option<usize> = None;

    // Главный рендеринг-цикл
    loop {
        // Время, прошедшее с предыдущего кадра (в секундах)
        let delta = get_frame_time();

        // Нажатие Escape — выход из программы
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // Нажатие C или Backspace — очистка всех точек и сброс состояния
        if is_key_pressed(KeyCode::C) || is_key_pressed(KeyCode::Backspace) {
            control_points.clear();
            is_animating = false;
            current_step = 0;
        }

        // Нажатие Enter — запуск анимации (требуется минимум 3 точки)
        if is_key_pressed(KeyCode::Enter) {
            if control_points.len() >= 3 {
                is_animating = true;
                current_step = 0;
                step_timer = 0.0;
            }
        }

        // Получение текущих координат курсора мыши
        let (mouse_x, mouse_y) = mouse_position();
        let mouse_pos = Point::new(mouse_x, mouse_y);

        // Обработка одиночного клика левой кнопкой мыши
        if is_mouse_button_pressed(MouseButton::Left) {
            let mut clicked_idx = None;

            // Проверяем, кликнул ли пользователь рядом с существующей точкой
            for (i, pt) in control_points.iter().enumerate() {
                let dx = pt.x - mouse_x;
                let dy = pt.y - mouse_y;
                // Радиус захвата точки — 15 пикселей (по теореме Пифагора)
                if (dx * dx + dy * dy).sqrt() < 15.0 {
                    clicked_idx = Some(i);
                    break;
                }
            }

            // Если кликнули по существующей точке — начинаем перетаскивание, иначе создаём новую
            if let Some(idx) = clicked_idx {
                dragged_point_index = Some(idx);
            } else {
                control_points.push(mouse_pos);
            }
        }

        // Обновление позиции перетаскиваемой точки при зажатой ЛКМ
        if is_mouse_button_down(MouseButton::Left) {
            if let Some(idx) = dragged_point_index {
                if idx < control_points.len() {
                    control_points[idx] = mouse_pos;
                }
            }
        } else {
            // Отпускание кнопки мыши завершает перетаскивание
            dragged_point_index = None;
        }

        // Логика обновления шагов анимации по таймеру
        if is_animating && control_points.len() >= 3 {
            step_timer += delta;
            if step_timer >= step_duration {
                step_timer = 0.0;
                current_step += 1;
                // Зацикливание анимации: после 7-го шага возвращаемся к исходным прямым линиям (шаг 0)
                if current_step > 7 {
                    current_step = 0;
                }
            }
        }

        // Очистка экрана чёрным цветом перед рисовкой нового кадра
        clear_background(BLACK);

        // Отрисовка линий
        if control_points.len() >= 2 {
            if !is_animating || control_points.len() == 2 || current_step == 0 {
                // Отрисовка исходной ломаной линии без сглаживания (Шаг 0 или режим редактирования)
                for i in 0..control_points.len() - 1 {
                    draw_line(
                        control_points[i].x,
                        control_points[i].y,
                        control_points[i + 1].x,
                        control_points[i + 1].y,
                        2.0,
                        WHITE,
                    );
                }
            } else {
                // Вычисление сглаженных точек для текущего шага
                let curve_points = chaikin_n_steps(&control_points, current_step);
                
                // 1. Соединение первой опорной точки с началом сглаженной кривой
                draw_line(
                    control_points[0].x,
                    control_points[0].y,
                    curve_points[0].x,
                    curve_points[0].y,
                    2.0,
                    WHITE,
                );

                // 2. Отрисовка промежуточных отрезков сглаженной кривой
                for i in 0..curve_points.len() - 1 {
                    draw_line(
                        curve_points[i].x,
                        curve_points[i].y,
                        curve_points[i + 1].x,
                        curve_points[i + 1].y,
                        2.0,
                        WHITE,
                    );
                }

                // 3. Соединение конца сглаженной кривой с последней опорной точкой
                let last_curve = curve_points.last().unwrap();
                let last_ctrl = control_points.last().unwrap();
                draw_line(
                    last_curve.x,
                    last_curve.y,
                    last_ctrl.x,
                    last_ctrl.y,
                    2.0,
                    WHITE,
                );
            }
        }

        // Отрисовка кружков вокруг всех опорных точек
        for pt in &control_points {
            draw_circle_lines(pt.x, pt.y, 6.0, 1.5, WHITE);
        }

        // Ожидание следующего кадра
        next_frame().await;
    }
}

// Модуль юнит-тестов для проверки корректности работы алгоритма
#[cfg(test)]
mod tests {
    use super::*;

    // Тест корректности математических расчётов координат алгоритма Чайкина
    #[test]
    fn test_chaikin_coordinates() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(100.0, 100.0),
        ];

        let result = chaikin_step(&points);

        assert_eq!(result.len(), 4);
        assert_eq!(result[0], Point::new(25.0, 0.0));
        assert_eq!(result[1], Point::new(75.0, 0.0));
        assert_eq!(result[2], Point::new(100.0, 25.0));
        assert_eq!(result[3], Point::new(100.0, 75.0));
    }

    // Тест формулы количества точек после одного шага (2N - 2)
    #[test]
    fn test_point_count_formula() {
        let p3 = vec![Point::new(0.0, 0.0), Point::new(10.0, 10.0), Point::new(20.0, 0.0)];
        assert_eq!(chaikin_step(&p3).len(), 4);

        let p5 = vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(20.0, 0.0),
            Point::new(30.0, 10.0),
            Point::new(40.0, 0.0),
        ];
        assert_eq!(chaikin_step(&p5).len(), 8);
    }

    // Тест граничных случаев (пустой вектор, 1 или 2 точки)
    #[test]
    fn test_edge_cases() {
        let empty: Vec<Point> = vec![];
        assert_eq!(chaikin_step(&empty), vec![]);

        let p1 = vec![Point::new(10.0, 10.0)];
        assert_eq!(chaikin_step(&p1), p1);

        let p2 = vec![Point::new(0.0, 0.0), Point::new(10.0, 10.0)];
        assert_eq!(chaikin_step(&p2), p2);
    }
}
/// ВАЖНО: все задания выполнять не обязательно. Что получится то получится сделать.

/// Задание 1
/// Почему фунция example1 зависает?
/// 
/// `a1` попадет в планировщик первой, поток у планировщика один, а метод try_recv синхронный,
/// поэтому до `a2` управление даже не дойдет.
fn example1() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .build()
        .unwrap();
    let (sd, mut rc) = tokio::sync::mpsc::unbounded_channel();

    let a1 = async move {
        loop {
            if let Ok(p) = rc.try_recv() {
                println!("{}", p);
                break;
            }

            println!("Hi");
        }
    };

    let h1 = rt.spawn(a1);

    let a2 = async move {
        let _ = sd.send("message");
    };
    let h2 = rt.spawn(a2);
    while !(h1.is_finished() || h2.is_finished()) {}

    println!("execution completed");
}

#[derive(Clone)]
struct Example2Struct {
    value: u64,
    ptr: *const u64,
}

/// Задание 2
/// Какое число тут будет распечатано 32 64 или 128 и почему?
/// 
/// Напечатается 64. На момент создания t2, его ptr будет указывать на t1.value.
/// drop(t1) ничего не затрет, потому что память и так освободится после завершения функции.
/// t2.ptr указывает на t1.value, на который ссылаться нельзя, но в памяти он все еще 
/// существует в неизменном виде. 
fn example2() {

    let num = 32;

    let mut t1 = Example2Struct {
        value: 64,
        ptr: &num,
    };

    t1.ptr = &t1.value;

    let mut t2 = t1.clone();

    drop(t1);

    t2.value = 128;

    unsafe {
        println!("{}", t2.ptr.read());
    }

    println!("execution completed");
}

/// Задание 3
/// Почему время исполнения всех пяти заполнений векторов разное (под linux)?
/// 
/// Разные паттерны работы с памятью
/// 1 - по мере заполнения вектора будут реалокации, а это очень грустно и долго
/// 2 - реалокаций не будет, поэтому быстрее
/// 3 - я думаю макрос развернется во второй вариант, только с push вместо insert по последнему индексу
/// 4 - просто итерация по вектору (даже не пишем ничего в него)
/// 5 - честно не знаю почему так быстро, но думаю что из-за 0 это не простой malloc, что-то более 
/// специализированное.
fn example3() {
    let capacity = 10000000u64;

    let start_time = std::time::Instant::now();
    let mut my_vec1 = Vec::new();
    for i in 0u64..capacity {
        my_vec1.insert(i as usize, i);
    }
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let start_time = std::time::Instant::now();
    let mut my_vec2 = Vec::with_capacity(capacity as usize);
    for i in 0u64..capacity {
        my_vec2.insert(i as usize, i);
    }
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let start_time = std::time::Instant::now();
    let mut my_vec3 = vec![6u64; capacity as usize];
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let start_time = std::time::Instant::now();
    for mut elem in my_vec3 {
        elem = 7u64;
    }
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let start_time = std::time::Instant::now();
    let my_vec4 = vec![0u64; capacity as usize];
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    println!("execution completed");
}

/// Задание 4
/// Почему такая разница во времени выполнения example4_async_mutex и example4_std_mutex?
/// 
/// В первом примере токийный примитив, который не отправляет системные треды спать на фьютексе(это 
/// соответственно делает std::sync::Mutex) в случае если он залочен, а
/// кладет в очередь ожидания примитива остановленную корутину. Разница в стоимости
/// suspend корутины и отправки целого потока(это еще и поток рантайма) на сон во фьютекс огромная. 
async fn example4_async_mutex(tokio_protected_value: std::sync::Arc<tokio::sync::Mutex<u64>>) {
    for _ in 0..1000000 {
        let mut value = *tokio_protected_value.clone().lock().await;
        value = 4;
    }
}

async fn example4_std_mutex(protected_value: std::sync::Arc<std::sync::Mutex<u64>>) {
    for _ in 0..1000000 {
        let mut value = *protected_value.clone().lock().unwrap();
        value = 4;
    }
}

fn example4() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .unwrap();

    let mut tokio_protected_value = std::sync::Arc::new(tokio::sync::Mutex::new(0u64));

    let start_time = std::time::Instant::now();
    let h1 = rt.spawn(example4_async_mutex(tokio_protected_value.clone()));
    let h2 = rt.spawn(example4_async_mutex(tokio_protected_value.clone()));
    let h3 = rt.spawn(example4_async_mutex(tokio_protected_value.clone()));

    while !(h1.is_finished() || h2.is_finished() || h3.is_finished()) {}
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    let protected_value = std::sync::Arc::new(std::sync::Mutex::new(0u64));

    let start_time = std::time::Instant::now();
    let h1 = rt.spawn(example4_std_mutex(protected_value.clone()));
    let h2 = rt.spawn(example4_std_mutex(protected_value.clone()));
    let h3 = rt.spawn(example4_std_mutex(protected_value.clone()));

    while !(h1.is_finished() || h2.is_finished() || h3.is_finished()) {}
    println!(
        "execution time {}",
        (std::time::Instant::now() - start_time).as_nanos()
    );

    println!("execution completed");
}

/// Задание 5
/// В чем ошибка дизайна? Каких тестов не хватает? Есть ли лишние тесты?
/// 
/// Ошибка в том, что достаточно просто сломать инварианты структуры.
/// Например, после Triangle::new - area - изменить поле - area последний 
/// вызов вернет неправильное значение. Приватные поля и публичные сеттеры могут решить эту проблему.
/// Лично мне такое не по душе, я бы просто запретил менять поля - Triangle иммутабельная структура
/// и чтобы поменять поля создайте новую с измененным полем.
/// 
/// Можно в конструктор сразу вершины передавать и либо убрал бы ленивость полей area, perimeter - сразу в конструкторе 
/// заполнять их, либо вообще убрал бы их из полей и вынес в методы, в общем - зависит от паттерна работы со структурой.
/// 
/// В тестах явно не хватает плохих значений для полей, кейса который я выше описал.
mod example5 {
    pub struct Triangle {
        pub a: (f32, f32),
        pub b: (f32, f32),
        pub c: (f32, f32),
        area: Option<f32>,
        perimeter: Option<f32>,
    }

    impl Triangle {
        //calculate area which is a positive number
        pub fn area(&mut self) -> f32 {
            if let Some(area) = self.area {
                area
            } else {
                self.area = Some(f32::abs(
                    (1f32 / 2f32) * (self.a.0 - self.c.0) * (self.b.1 - self.c.1)
                        - (self.b.0 - self.c.0) * (self.a.1 - self.c.1),
                ));
                self.area.unwrap()
            }
        }

        fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
            f32::sqrt((b.0 - a.0) * (b.0 - a.0) + (b.1 - a.1) * (b.1 - a.1))
        }

        //calculate perimeter which is a positive number
        pub fn perimeter(&mut self) -> f32 {
            if let Some(perimeter) = self.perimeter {
                return perimeter;
            } else {
                self.perimeter = Some(
                    Triangle::dist(self.a, self.b)
                        + Triangle::dist(self.b, self.c)
                        + Triangle::dist(self.c, self.a),
                );
                self.perimeter.unwrap()
            }
        }

        //new makes no guarantee for a specific values of a,b,c,area,perimeter at initialization
        pub fn new() -> Triangle {
            Triangle {
                a: (0f32, 0f32),
                b: (0f32, 0f32),
                c: (0f32, 0f32),
                area: None,
                perimeter: None,
            }
        }
    }
}

#[cfg(test)]
mod example5_tests {
    use super::example5::Triangle;

    #[test]
    fn test_area() {
        let mut t = Triangle::new();

        t.a = (0f32, 0f32);
        t.b = (0f32, 0f32);
        t.c = (0f32, 0f32);

        assert!(t.area() == 0f32);

        let mut t = Triangle::new();

        t.a = (0f32, 0f32);
        t.b = (0f32, 1f32);
        t.c = (1f32, 0f32);

        assert!(t.area() == 0.5);

        let mut t = Triangle::new();

        t.a = (0f32, 0f32);
        t.b = (0f32, 1000f32);
        t.c = (1000f32, 0f32);

        println!("{}",t.area());
    }

    #[test]
    fn test_perimeter() {
        let mut t = Triangle::new();

        t.a = (0f32, 0f32);
        t.b = (0f32, 0f32);
        t.c = (0f32, 0f32);

        assert!(t.perimeter() == 0f32);

        let mut t = Triangle::new();

        t.a = (0f32, 0f32);
        t.b = (0f32, 1f32);
        t.c = (1f32, 0f32);

        assert!(t.perimeter() == 2f32 + f32::sqrt(2f32));
    }
}

fn main() {
    example1();
}
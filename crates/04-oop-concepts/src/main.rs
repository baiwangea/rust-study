// 定义一个共享行为的 Trait
pub trait Drawable {
    fn draw(&self);
}

// 定义一个屏幕 Struct，它可以持有一系列可绘制的组件
// Box<dyn Drawable> 是一个 trait object，它允许在运行时持有不同类型但实现了相同 trait 的实例
pub struct Screen {
    pub components: Vec<Box<dyn Drawable>>,
}

impl Screen {
    pub fn run(&self) {
        println!("开始在屏幕上绘制所有组件...");
        for component in self.components.iter() {
            component.draw();
        }
        println!("绘制完成！");
    }
}

// 定义一个按钮 Struct
#[derive(Debug)]
pub struct Button {
    pub width: u32,
    pub height: u32,
    pub label: String,
}

// 为 Button 实现 Drawable Trait
impl Drawable for Button {
    fn draw(&self) {
        println!("绘制一个按钮: {:?}", self);
    }
}

// 定义另一个 Struct，比如一个文本输入框
#[derive(Debug)]
#[allow(dead_code)] // 允许字段不被读取，以消除警告
struct SelectBox {
    width: u32,
    height: u32,
    options: Vec<String>,
}

// 为 SelectBox 实现 Drawable Trait
impl Drawable for SelectBox {
    fn draw(&self) {
        println!("绘制一个选择框: {:?}", self);
    }
}

fn main() {
    let screen = Screen {
        components: vec![
            Box::new(SelectBox {
                width: 75,
                height: 10,
                options: vec![
                    String::from("Yes"),
                    String::from("No"),
                    String::from("Maybe"),
                ],
            }),
            Box::new(Button {
                width: 50,
                height: 10,
                label: String::from("OK"),
            }),
        ],
    };

    screen.run();
}

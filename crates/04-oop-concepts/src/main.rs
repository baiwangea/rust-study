//! 面向对象思想在 Rust 中的体现。
//!
//! - Trait + trait object (`Box<dyn Trait>`)：运行时多态（动态分发）
//! - 泛型 + `impl Trait`：编译期多态（静态分发，零成本抽象）
//! - enum + 移动语义：类型状态模式（typestate），用类型系统保证状态流转合法

// ==================== 1. 动态分发：trait object ====================

/// 共享行为的 Trait
pub trait Drawable {
    fn draw(&self);
}

/// 屏幕可以持有一系列"不同类型但都可绘制"的组件，
/// `Box<dyn Drawable>` 是 trait object，通过虚函数表在运行时分发
pub struct Screen {
    pub components: Vec<Box<dyn Drawable>>,
}

impl Screen {
    pub fn run(&self) {
        println!("开始在屏幕上绘制所有组件...");
        for component in &self.components {
            component.draw();
        }
        println!("绘制完成！");
    }
}

#[derive(Debug)]
pub struct Button {
    pub width: u32,
    pub height: u32,
    pub label: String,
}

impl Drawable for Button {
    fn draw(&self) {
        println!("绘制一个按钮: {:?}", self);
    }
}

#[derive(Debug)]
pub struct SelectBox {
    pub width: u32,
    pub height: u32,
    pub options: Vec<String>,
}

impl Drawable for SelectBox {
    fn draw(&self) {
        println!("绘制一个选择框: {:?}", self);
    }
}

// ==================== 2. 静态分发：泛型单态化 ====================

/// `impl Trait` 参数：编译器为每种具体类型生成一份专用代码（单态化），
/// 没有虚函数表开销，但无法在同一个容器里混装不同类型
fn draw_static(item: &impl Drawable) {
    print!("[静态分发] ");
    item.draw();
}

/// 对比：`&dyn Trait` 走运行时查表，灵活但有少量间接调用开销
fn draw_dynamic(item: &dyn Drawable) {
    print!("[动态分发] ");
    item.draw();
}

// ==================== 3. 类型状态模式：订单状态机 ====================
// 每个状态是独立的类型，状态流转通过"消费自身、返回新类型"的方法实现。
// 非法流转（例如未支付就发货）在编译期就被拒绝，无需运行时检查。

pub struct DraftOrder {
    items: Vec<String>,
}

impl DraftOrder {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// 草稿阶段可以反复添加商品（返回 self 支持链式调用）
    pub fn add_item(mut self, item: &str) -> Self {
        self.items.push(item.to_string());
        self
    }

    /// 提交后 DraftOrder 被消费，无法再修改——类型系统保证这一点
    pub fn submit(self) -> SubmittedOrder {
        println!("订单已提交，包含 {} 件商品", self.items.len());
        SubmittedOrder { items: self.items }
    }
}

impl Default for DraftOrder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SubmittedOrder {
    items: Vec<String>,
}

impl SubmittedOrder {
    pub fn pay(self) -> PaidOrder {
        println!("支付成功");
        PaidOrder { items: self.items }
    }
}

pub struct PaidOrder {
    items: Vec<String>,
}

impl PaidOrder {
    pub fn ship(self) {
        println!("发货：共 {} 件商品", self.items.len());
    }
}

fn main() {
    // --- 1. 动态分发 ---
    let screen = Screen {
        components: vec![
            Box::new(SelectBox {
                width: 75,
                height: 10,
                options: vec![String::from("Yes"), String::from("No")],
            }),
            Box::new(Button {
                width: 50,
                height: 10,
                label: String::from("OK"),
            }),
        ],
    };
    screen.run();

    // --- 2. 静态分发与动态分发对比 ---
    println!("\n--- 静态分发 vs 动态分发 ---");
    let button = Button {
        width: 80,
        height: 20,
        label: String::from("Submit"),
    };
    draw_static(&button);
    draw_dynamic(&button);

    // --- 3. 类型状态模式 ---
    println!("\n--- 类型状态模式：订单流转 ---");
    DraftOrder::new()
        .add_item("机械键盘")
        .add_item("鼠标垫")
        .submit() // DraftOrder -> SubmittedOrder
        .pay() // SubmittedOrder -> PaidOrder
        .ship(); // PaidOrder -> ()
    // 反例：`DraftOrder::new().ship()` 无法编译，草稿订单不能直接发货
}

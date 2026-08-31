//! 常用数据结构示例：标准库集合 + 手写单链表。
//!
//! 覆盖：Vec、HashMap、VecDeque、BTreeMap/BTreeSet、BinaryHeap，
//! 以及一个展示所有权驱动设计的手写泛型链表（含迭代器与迭代式 Drop）。

use std::collections::{BinaryHeap, BTreeMap, BTreeSet, HashMap, VecDeque};
use std::cmp::Reverse;

fn main() {
    vec_demo();
    hashmap_demo();
    vecdeque_demo();
    btree_demo();
    binary_heap_demo();
    linked_list_demo();
}

fn vec_demo() {
    println!("--- Vec (动态数组) ---");
    let mut numbers = vec![1, 2, 3];
    numbers.push(4);
    println!("Vec: {:?}", numbers);
    // `get` 返回 Option，越界时不会 panic，比下标访问更安全
    println!("第一个元素: {:?}, 越界访问: {:?}", numbers.get(0), numbers.get(99));
}

fn hashmap_demo() {
    println!("\n--- HashMap (哈希图) ---");
    let mut scores = HashMap::new();
    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);

    // entry API：插入"若不存在"的默认值，避免二次查找
    scores.entry(String::from("Blue")).or_insert(0);
    scores.entry(String::from("Green")).or_insert(5);

    let score = scores.get("Blue").copied().unwrap_or(0);
    println!("Blue 队的分数: {}", score);
    for (team, score) in &scores {
        println!("{}: {}", team, score);
    }
}

fn vecdeque_demo() {
    println!("\n--- VecDeque (双端队列) ---");
    // 两端都是 O(1) 的插入/弹出，适合实现队列
    let mut queue = VecDeque::new();
    queue.push_back("任务1");
    queue.push_back("任务2");
    queue.push_front("插队任务"); // 插到队首
    while let Some(item) = queue.pop_front() {
        println!("出队: {}", item);
    }
}

fn btree_demo() {
    println!("\n--- BTreeMap / BTreeSet (有序集合) ---");
    // BTreeMap 按键有序（HashMap 无序），支持范围查询
    let mut ages = BTreeMap::new();
    ages.insert("alice", 30);
    ages.insert("bob", 25);
    ages.insert("carol", 35);
    println!("按名字排序遍历: {:?}", ages);

    // BTreeSet：有序去重集合
    let tags: BTreeSet<&str> = ["rust", "web", "rust", "async"].into_iter().collect();
    println!("有序去重: {:?}", tags);
}

fn binary_heap_demo() {
    println!("\n--- BinaryHeap (优先队列) ---");
    // 默认是最大堆
    let mut max_heap = BinaryHeap::new();
    for n in [3, 1, 4, 1, 5] {
        max_heap.push(n);
    }
    println!("最大堆弹出顺序: {:?}", (0..3).map(|_| max_heap.pop().unwrap()).collect::<Vec<_>>());

    // 用 Reverse 包装得到最小堆
    let mut min_heap: BinaryHeap<Reverse<i32>> = BinaryHeap::new();
    for n in [3, 1, 4, 1, 5] {
        min_heap.push(Reverse(n));
    }
    let first: Vec<i32> = (0..3).map(|_| min_heap.pop().unwrap().0).collect();
    println!("最小堆弹出顺序: {:?}", first);
}

// --- 手写单链表：所有权驱动的数据结构设计 ---

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    value: T,
    next: Link<T>,
}

pub struct LinkedList<T> {
    head: Link<T>,
    len: usize,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self { head: None, len: 0 }
    }

    pub fn push_front(&mut self, value: T) {
        let new_node = Box::new(Node {
            value,
            next: self.head.take(), // take 把 head 置空并交出所有权，避免移动已借用的字段
        });
        self.head = Some(new_node);
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            self.len -= 1;
            node.value
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            current: self.head.as_deref(),
        }
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for LinkedList<T> {
    // 迭代式 Drop：递归实现在长链表上会栈溢出，这里循环取走每个节点
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

pub struct Iter<'a, T> {
    current: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|node| {
            self.current = node.next.as_deref();
            &node.value
        })
    }
}

fn linked_list_demo() {
    println!("\n--- 手写单链表 ---");
    let mut list = LinkedList::new();
    list.push_front(3);
    list.push_front(2);
    list.push_front(1);
    println!("长度: {}", list.len());
    println!("迭代: {:?}", list.iter().copied().collect::<Vec<_>>());
    println!("弹出: {:?}, {:?}", list.pop_front(), list.pop_front());
}

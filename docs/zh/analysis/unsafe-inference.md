# Unsafe 类型推断引擎

> 基于启发式的类型检测，用于 FFI 分配和裸指针

---

## 概述

**文件:** `src/analysis/unsafe_inference/engine.rs` (670 行)

当类型信息不可用时 — 例如 FFI 分配、裸指针或通过 `malloc`/`calloc` 获取的内存 — 推断引擎使用**多维度启发式分析**来猜测可能的类型。

它操作的是原始字节模式，而非 Rust 类型元数据，因此适用于任何内存区域，无论其来源。

---

## 设计：六维评分

引擎使用**评分模型**，六个独立维度各自为不同的类型假设贡献证据。聚合得分最高的类型胜出。

```
                    ┌──────────────────────┐
                    │   评分聚合器         │
                    │   max(score) → 猜测  │
                    └──────────┬───────────┘
         ┌─────────┬──────────┼──────────┬─────────┐
         ▼         ▼          ▼          ▼         ▼
      大小     布局检测    内容分析    指针计数   2 的幂
     启发式    (O(1))      (O(n))      (O(n))    容量 (O(1))
    (O(1))
```

### 内部评分结构

```rust
// engine.rs:126-135
#[derive(Default)]
struct Score {
    vec: u8,       // Vec<T> 的证据
    string: u8,    // String 的证据
    cstring: u8,   // C 风格 null 终止字符串的证据
    pointer: u8,   // 裸指针 (*mut T, *const T) 的证据
    fat_ptr: u8,   // 胖指针 (&[T], &str, dyn Trait) 的证据
    buffer: u8,    // 原始字节缓冲区 ([u8]) 的证据
    cstruct: u8,   // 带指针字段的 C 结构体的证据
}
```

---

## 维度一：大小启发

**文件:** `engine.rs:226-255`

根据常见 Rust/C 类型布局，将分配大小映射到可能的类型类别：

```rust
fn size_heuristic(size: usize, score: &mut Score) {
    match size {
        // 裸指针: *mut T, *const T, &T, Box<T>
        // 从 60 降至 30 以减少误判
        8 => score.pointer += 30,

        // 胖指针: &[T], &str, dyn Trait (data_ptr + metadata)
        16 => score.fat_ptr += 25,

        // Vec/String 三元组: (ptr, len, cap) — 3 × usize = 24 字节
        24 => {
            score.vec += 15;
            score.string += 15;
        }

        // 常见 C 结构体大小
        32 | 48 | 64 => score.cstruct += 10,

        _ => {}
    }

    // 2 的幂信号: Rust Vec 容量按 2 的幂增长
    // 随机大小恰好是 2 的幂的概率 ≈ 1/size
    // 对 64 字节而言，误判率仅 ~1.5%
    if size.is_power_of_two() && size >= 64 {
        score.vec += 10;
        score.buffer += 5;
    }
}
```

**为什么 2 的幂是强信号：** Rust 的 `Vec` 增长策略是 `new_cap = max(old_cap * 2, 1)`。所以实际堆缓冲区大小几乎总是 2 的幂。

---

## 维度二：Vec/String 布局检测

**文件:** `engine.rs:261-303`

检测 `Vec<T>` 和 `String` 共有的 `(ptr, len, cap)` 三元组结构：

```rust
fn vec_string_layout(view: &MemoryView, score: &mut Score) {
    if view.len() < 24 { return; }

    let ptr_val = view.read_usize(0);
    let len     = view.read_usize(8);
    let cap     = view.read_usize(16);

    let (Some(p), Some(l), Some(c)) = (ptr_val, len, cap) else { return; };

    // 基本结构验证
    if !is_valid_ptr(p) || c < l || c == 0 || c > 10_000_000 { return; }

    // 根据备用容量区分 Vec 和 String
    let spare = c.saturating_sub(l);

    if spare < 16 && l > 0 {
        // 备用容量小 → 更可能是 String
        score.string += 50;
        score.vec += 20;
    } else if spare > 0 {
        // 备用容量大 → 更可能是 Vec (预分配增长空间)
        score.vec += 60;
        score.string += 15;
    } else {
        // cap == len → 可能是任意一种
        score.vec += 30;
        score.string += 30;
    }

    // 额外 Vec 信号: 容量是 2 的幂
    if c.is_power_of_two() {
        score.vec += 15;
    }
}
```

**如何区分 Vec 和 String：**
- `String` 通常 `cap` 接近 `len`（备用容量小），因为字符串是增量构建的
- `Vec` 通常 `cap >> len`（备用容量大），由于预分配增长策略

---

## 维度三：内容分析

**文件:** `engine.rs:308-438`

对原始内存内容进行三个子分析：

### 3a. 增强 CString 检测

```rust
fn cstring_enhanced(data: &[u8], score: &mut Score) {
    // 找到第一个 null 字节
    let null_pos = match data.iter().position(|&b| b == 0) {
        Some(pos) => pos,
        None => return,  // 无 null 终止符
    };

    if null_pos < 3 { return; }  // 太短

    let content = &data[..null_pos];

    // 统计可打印 ASCII 字符 (0x20-0x7E)
    let printable_count = content.iter()
        .filter(|&&b| (0x20..=0x7E).contains(&b))
        .count();
    let printable_ratio = printable_count as f32 / content.len() as f32;

    // 高可打印比例 → 可能是 CString
    if printable_ratio > 0.9 { score.cstring += 70; }
    else if printable_ratio > 0.7 { score.cstring += 40; }
    else if printable_ratio > 0.5 { score.cstring += 20; }

    // 多个 null → 可能是二进制数据，不是 CString
    let null_count = data.iter().filter(|&&b| b == 0).count();
    if null_count > 1 {
        score.cstring = score.cstring.saturating_sub(20);
        score.buffer += 15;
    }
}
```

### 3b. Shannon 熵分析

```rust
fn entropy_analysis(data: &[u8], score: &mut Score) {
    let entropy = shannon_entropy(data);

    // 高熵 → 压缩/加密/序列化数据
    if entropy > 7.5 { score.buffer += 30; }
    else if entropy > 6.5 { score.buffer += 15; }
    // 低熵 → 重复数据或文本
    else if entropy < 3.0 { score.cstruct += 5; }
}
```

**典型熵值：**

| 数据类型 | 熵值范围 |
|----------|----------|
| 英文文本 | 4.0-4.5 |
| 源代码 | 4.5-5.0 |
| 压缩数据 | 7.8-8.0 |
| 加密数据 | 7.9-8.0 |
| 指针数组 | 3.0-5.0 |
| 零填充 | 0.0 |

### 3c. 零填充检测

```rust
fn zero_fill_detection(data: &[u8], score: &mut Score) {
    if data.len() < 16 { return; }
    let zero_ratio = data.iter().filter(|&&b| b == 0).count() as f32 / data.len() as f32;
    if zero_ratio > 0.9 {
        score.buffer += 15;
        score.cstruct += 10;  // 零填充的结构体 padding
    }
}
```

---

## 维度四：指针启发

**文件:** `engine.rs:443-457`

统计内存区域中的有效指针数量，以区分缓冲区和 C 结构体：

```rust
fn pointer_heuristic(view: &MemoryView, score: &mut Score) {
    let ptr_count = count_valid_pointers(view);

    if ptr_count == 0 && view.len() > 8 {
        // 无有效指针 → 可能是缓冲区
        score.buffer += 40;
    } else if ptr_count == 1 {
        // 单个指针 → 可能是 Box 或简单结构体
        score.pointer += 10;
        score.cstruct += 5;
    } else if ptr_count >= 2 {
        // 多个指针 → 可能是 C 结构体
        score.cstruct += 30;
    }
}
```

### 指针验证

**文件:** `memory_view.rs:58-60`

```rust
pub fn is_valid_ptr(p: usize) -> bool {
    p > MIN_VALID_ADDR && p < MAX_USER_ADDR
}
// MIN_VALID_ADDR = 0x1000 (第一页，OS 保留)
// MAX_USER_ADDR = 0x7fff_ffff_ffff (64 位上 128TB)
```

通过以 8 字节块扫描内存来统计有效指针：

```rust
// memory_view.rs:62-76
pub fn count_valid_pointers(view: &MemoryView) -> usize {
    let mut count = 0;
    for chunk in view.chunks(8) {
        if chunk.len() < 8 { break; }
        let mut buf = [0u8; 8];
        buf.copy_from_slice(chunk);
        let v = usize::from_le_bytes(buf);
        if is_valid_ptr(v) { count += 1; }
    }
    count
}
```

---

## 聚合：最终判定

**文件:** `engine.rs:460-479`

```rust
fn finalize(score: Score) -> TypeGuess {
    let table = [
        (TypeKind::Vec, score.vec),
        (TypeKind::String, score.string),
        (TypeKind::CString, score.cstring),
        (TypeKind::Pointer, score.pointer),
        (TypeKind::FatPtr, score.fat_ptr),
        (TypeKind::Buffer, score.buffer),
        (TypeKind::CStruct, score.cstruct),
    ];

    let mut best = (TypeKind::Unknown, 0u8);
    for (kind, val) in table {
        if val > best.1 { best = (kind, val); }
    }

    TypeGuess::new(best.0, best.1)
}
```

选择聚合得分最高的类型。置信度是原始得分值（0-100+）。

---

## 公共 API

```rust
pub struct UnsafeInferenceEngine;

impl UnsafeInferenceEngine {
    /// 从单个内存视图推断类型
    pub fn infer_single(view: &MemoryView, size: usize) -> TypeGuess;

    /// 从原始字节推断类型
    pub fn infer_from_bytes(data: &[u8], size: usize) -> TypeGuess;

    /// 对多条记录运行推断
    pub fn run(records: &mut [InferenceRecord]);
}
```

### 使用示例

```rust
use memscope_rs::analysis::unsafe_inference::{
    UnsafeInferenceEngine, TypeKind,
};

let memory = /* 来自 FFI 分配的原始字节 */;
let guess = UnsafeInferenceEngine::infer_from_bytes(&memory, size);
println!("可能类型: {} ({}% 置信度)", guess.kind, guess.confidence);
```

---

## TypeGuess 结构

```rust
pub struct TypeGuess {
    pub kind: TypeKind,       // 推断的类型类别
    pub confidence: u8,       // 置信度得分 (0-100+)
    pub method: InferenceMethod,  // 推断方式
}

pub enum TypeKind {
    Vec,       // Rust Vec<T>
    String,    // Rust String
    CString,   // C 风格 null 终止字符串
    Pointer,   // 裸指针 (*mut T, *const T, Box<T>)
    FatPtr,    // 胖指针 (&[T], &str, dyn Trait)
    Buffer,    // 原始字节缓冲区 ([u8])
    CStruct,   // 带多个指针字段的 C 结构体
    Unknown,   // 无法确定
}
```

---

## 性能

| 指标 | 值 |
|------|-----|
| 复杂度 | 每分配 O(n)，n = 内存大小 |
| 典型运行时间 | 100 万分配约 5-50ms |
| 运行时机 | 仅在快照分析期间（非追踪热路径） |
| 内存成本 | 极低 — 操作现有内存快照 |

---

## 准确率估算

| 类型 | 预期准确率 | 关键信号 |
|------|-----------|----------|
| Vec | ~70-85% | 大小=24, 2 的幂容量, 大备用 |
| String | ~60-80% | 大小=24, 小备用, (UTF-8 规划中) |
| CString | ~65-80% | Null 终止符, 高可打印 ASCII |
| Pointer | ~60-75% | 大小=8, 单个有效指针 |
| Buffer | ~60-75% | 零指针, 高熵 |
| CStruct | ~50-65% | 多个指针, 常见大小 |

---

## 已知限制

1. **尚无 UTF-8 验证** — 设计文档指定了 UTF-8 验证用于 String 检测，但尚未实现。String 与 Vec 的区分仅依赖备用容量。
2. **指针验证范围过宽** — `is_valid_ptr` 接受 0x1000 到 0x7fff_ffff_ffff（128TB）之间的任何地址，导致假阳性。
3. **无调用栈集成** — 设计文档指定使用调用栈符号作为类型提示，但尚未实现。
4. **无生命周期分析** — 设计文档指定使用分配生命周期模式，但尚未实现。
5. **合成数据偏差** — 全部 19 个测试使用人工构造的字节数组。真实世界准确率可能更低。

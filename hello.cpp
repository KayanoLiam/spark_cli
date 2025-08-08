# C++ Hello World 最小示例

以下是一个最简化的 C++ "Hello World" 程序实现，包含详细注释：

```cpp
/*
 * 最简单的C++ Hello World程序
 * 编译: g++ -o helloworld helloworld.cpp
 * 运行: ./helloworld
 */

// 包含标准输入输出流头文件
// 提供cout等I/O功能
#include <iostream>

// 使用标准命名空间
// 避免每次都要写std::前缀
using namespace std;

// 程序主函数
// 每个C++程序必须有一个main函数作为入口点
int main() {
    // 使用标准输出流打印字符串到控制台
    // endl表示换行并刷新缓冲区
    cout << "Hello, World!" << endl;
    
    // main函数返回0表示程序成功执行
    return 0;
}
```

## 使用说明

1. **保存文件**：将上述代码保存为`helloworld.cpp`

2. **编译程序**：
   - 使用g++编译器：
     ```bash
     g++ -o helloworld helloworld.cpp
     ```
   - 使用clang++编译器：
     ```bash
     clang++ -o helloworld helloworld.cpp
     ```

3. **运行程序**：
   ```bash
   ./helloworld
   ```

4. **预期输出**：
   ```
   Hello, World!
   ```

## 依赖说明

- 本程序仅需要标准C++库中的`<iostream>`头文件
- 需要安装C++编译器，如：
  - GCC (g++)
  - Clang (clang++)
  - Microsoft Visual C++ (MSVC)

这是最基本的C++程序结构，展示了：
1. 如何包含头文件
2. 如何使用命名空间
3. main函数的结构
4. 基本的输出操作
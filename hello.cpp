/**
 * 最小化 C++ Hello World 示例
 * 编译依赖：支持 C++11 及以上标准的编译器
 * 
 * 使用方式：
 * 1. 保存为 hello.cpp
 * 2. 编译命令: g++ -std=c++11 -o hello hello.cpp
 * 3. 运行: ./hello
 */

// 包含标准输入输出流头文件
// 这是C++标准库中处理控制台输入输出的主要组件
#include <iostream>

// 使用标准命名空间，避免每次都要写 std::
// 注：在大型项目中建议显式使用 std:: 前缀
using namespace std;

// 程序主入口
int main() {
    // 向标准输出流打印信息
    // cout = character output stream
    // endl = 结束行并刷新缓冲区
    cout << "Hello, World!" << endl;
    
    // 返回0表示程序正常退出
    // 在C/C++中，非0返回值通常表示错误
    return 0;
}

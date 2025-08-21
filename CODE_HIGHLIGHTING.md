# SPICE电路文件代码高亮实现

## 概述

本项目为SPICE电路文件(.cir)提供了完整的代码高亮支持，包括语法定义、主题配置和语言服务器集成。

## 实现架构

### 1. TextMate语法定义

#### 核心文件
- `cir.tmLanguage.json` - 主要的语法定义文件
- `syntaxes/cir.tmLanguage.json` - 备用语法定义

#### 语法模式匹配

| 语法类型 | 正则表达式 | 说明 |
|---------|-----------|------|
| 注释 | `^\\*.*$` | 匹配以*开头的注释行 |
| 控制关键字 | `^\\.(TRAN\|END\|IC\|LIB\|SUBCKT\|ENDS\|...)` | SPICE控制命令 |
| 组件标识符 | `^[RCLVIUXYZ]\\w*` | 电路组件（电阻、电容、电感等） |
| 节点标识符 | `\\bN\\d+\\b` | 节点编号 |
| 数值常量 | `\\b\\d+(\\.\\d+)?([eE][+-]?\\d+)?[munpfkgt]?` | 数值和单位 |
| 参数关键字 | `\\b(AC\|SIN\|DC\|PULSE\|EXP\|...)` | 信号源参数 |

### 2. 主题配置

#### 深色主题 (`themes/spice-dark-theme.json`)
- 专门为SPICE电路文件设计的深色主题
- 提供清晰的语法元素区分
- 支持所有语法作用域的颜色映射

#### 颜色映射
- **注释**: 绿色 (#6a9955)
- **字符串**: 橙色 (#ce9178)
- **关键字**: 蓝色 (#569cd6)
- **函数**: 黄色 (#dcdcaa)
- **变量**: 浅蓝色 (#9cdcfe)
- **参数**: 金色 (#ffd700)
- **数值**: 浅绿色 (#b5cea8)

### 3. 扩展配置

#### package.json配置
```json
{
  "grammars": [
    {
      "language": "cir",
      "scopeName": "source.cir",
      "path": "./cir.tmLanguage.json"
    }
  ],
  "themes": [
    {
      "label": "SPICE Dark Theme",
      "uiTheme": "vs-dark",
      "path": "./themes/spice-dark-theme.json"
    }
  ]
}
```

## 语法高亮特性

### 1. SPICE特定语法支持
- **控制命令**: `.TRAN`, `.END`, `.IC`, `.LIB`, `.SUBCKT`, `.ENDS`
- **组件类型**: R(电阻), C(电容), L(电感), V(电压源), I(电流源), U(子电路)
- **信号源**: AC, SIN, DC, PULSE, EXP, SFFM等
- **模型参数**: IS, RS, N, TT, CJO, VJ等

### 2. 数值和单位支持
- 支持科学计数法: `1.5e-6`
- 支持SPICE单位: `m`(毫), `u`(微), `n`(纳), `p`(皮), `f`(飞), `k`(千), `g`(吉), `t`(太)

### 3. 注释支持
- 单行注释: `* 这是注释`
- 块注释: `** 块注释 **`

## 使用示例

### 典型的SPICE电路文件
```spice
* 这是一个SPICE电路文件示例
* 包含各种语法元素

R1 N1 N2 10.0k
C1 N2 N3 1.0uF
L1 N3 N4 10.0mH

V1 N1 0 DC 5.0 AC 1.0 SIN(0 5 1k)
I1 N4 0 PULSE(0 1 0 1n 1n 1m 2m)

.MODEL D1 D(IS=1e-12 N=1.0)
.SUBCKT AMP IN OUT
R1 IN OUT 10k
.ENDS

.TRAN 1u 10m
.END
```

## 扩展开发

### 添加新的语法模式
1. 在`cir.tmLanguage.json`中添加新的pattern
2. 定义合适的scope name
3. 在主题文件中添加对应的颜色映射

### 自定义主题
1. 修改`themes/spice-dark-theme.json`
2. 调整颜色值以匹配您的偏好
3. 重新加载扩展以应用更改

## 调试和测试

### 查看语法作用域
1. 打开.cir文件
2. 按Ctrl+Shift+P
3. 输入"Developer: Inspect Editor Tokens and Scopes"
4. 点击任意位置查看语法作用域

### 测试语法规则
1. 创建测试.cir文件
2. 包含各种语法元素
3. 验证高亮效果是否符合预期

## 技术细节

### TextMate语法引擎
- 使用正则表达式进行模式匹配
- 支持嵌套和递归语法规则
- 提供丰富的语法作用域

### VS Code集成
- 通过package.json配置语法和主题
- 支持动态语法加载
- 提供实时语法高亮

## 未来改进

1. **更精确的语法规则**: 添加更多SPICE特定的语法模式
2. **语义高亮**: 集成LSP服务器提供语义高亮
3. **代码片段**: 添加常用的SPICE代码片段
4. **错误检测**: 集成语法错误检测和提示
5. **自动完成**: 提供智能的代码自动完成功能 
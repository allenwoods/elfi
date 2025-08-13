# 测试场景 03: 文档即 App (Document as App)

本测试旨在验证 `elfi` 的 Recipe 系统在跨文档内容组合与动态引用方面的能力，这是构建复杂应用的基础。

- **核心目标**: 证明通过 Recipe 系统，一个主 `.elf` 文件可以引用并动态地组合其他 `.elf` 文件中的区块内容，实现类似"文档即App"的效果。

## 📁 关联的测试文件

**[main.elf](./main.elf)** - 主应用文档

这个文件展示了跨文档引用的完整工作流程：
- `intro`: 应用介绍和架构说明
- `placeholder-utils`: Link类型区块，引用外部组件
- `dynamic-composition`: Recipe配置，定义跨文档组合规则
- `test-scenarios`: 完整的测试场景定义

**[component.elf](./component.elf)** - 可复用组件库

这个文件包含被引用的工具和组件：
- `reusable-utilities`: Python工具函数库（被main.elf引用）
- `data-structures`: 核心数据结构定义
- `algorithms`: 文档处理算法
- `component-documentation`: 详细的API文档

### 跨文档引用关系

```
main.elf                    component.elf
├── placeholder-utils  ──→  ├── reusable-utilities
├── dynamic-composition     ├── data-structures
└── test-scenarios          └── algorithms
```

## 关联的实现文档

- `implementations/04-recipe_system.md`: Recipe 系统的跨文档引用和内容解析机制。
- `implementations/02-core_logic.md`: `Repo` 需要能够根据 URI 获取其他文档的 `DocHandle`。
- `implementations/03-cli.md`: `export` 命令对 Recipe 系统的支持，包括引用解析和错误处理。

## 测试流程设计

1.  **准备 (Preparation)**
    -   创建两个 `.elf` 文档实例：
        -   `component.elf`: 包含一个可复用的 `code` 区块，ID 为 `reusable-utilities`，内容为简单的 Python 工具函数。
        -   `main.elf`: 包含以下区块：
            - `intro` (类型: markdown): 项目介绍
            - `placeholder-utils` (类型: markdown): 占位符区块，将通过Recipe更换为引用内容
            - `usage-guide` (类型: markdown): 使用说明
            - `dynamic-composition` (类型: recipe): 动态内容组合配置

2.  **执行 (Execution)**
    -   使用 Recipe 系统导出组合后的内容：
        ```bash
        # 使用自定义的dynamic-composition Recipe
        elfi export --recipe=dynamic-composition ./output/
        ```
        Recipe 将会：
        - 选择所有 markdown 类型的区块
        - 将 `placeholder-utils` 区块更换为来自 `component.elf` 中 `reusable-utilities` 的实际内容
        - 生成一个组合后的 markdown 文件

3.  **验证 (Verification)**
    -   **初次组合**: 检查 `main.elf` 导出的结果。确认 `placeholder-utils` 区块没有被原样输出，而是被 `component.elf` 中 `reusable-utilities` 区块的实际内容所替换。
    -   **动态更新**: 修改 `component.elf` 中 `reusable-utilities` 区块的内容，例如添加新的工具函数。
    -   再次执行 Recipe 导出命令。
    -   检查 `main.elf` 的新导出结果，确认它反映了 `component.elf` 中更新后的内容。
    -   **错误处理测试**: 
        - 修改Recipe中的引用为不存在的区块：`elf://my-project/component/nonexistent-block`
        - 验证系统按照配置的错误处理策略处理（占位符/错误/跳过）
    -   **循环引用测试**: 创建A引用B、B引用A的情况，验证系统能够检测并报错。

## 成功标准

-   `main.elf` 的导出结果总是能正确地、动态地反映 `component.elf` 中被引用区块的最新状态。
-   Recipe 系统能够正确解析跨文档引用，实现动态内容组合。
-   错误处理机制能够优雅地处理各种异常情况：
    - 缺失引用：按照配置的策略处理（占位符/错误/跳过）
    - 循环引用：检测并阻止无限递归
    - URI格式错误：提供清晰的错误信息
-   系统表现出良好的鲁棒性和可靠性，证明了“文档即App”的可行性。

## Recipe 配置示例

为了实现上述测试，`dynamic-composition` 区块的内容应该包含类似以下的 YAML 配置：

```yaml
name: dynamic-composition
version: 1.0
description: 动态组合跨文档内容

# 跨文档引用配置
references:
  - source: "elf://my-project/component/reusable-utilities"
    target_block: "placeholder-utils"
    template: |
      ## 共享工具
      以下是来自 component.elf 的实用工具：
      
      ```python
      {resolved_content}
      ```

# 选择器：处理所有 markdown 区块
selector:
  types: [markdown]

# 转换规则
transform:
  - type: markdown
    action: resolve_references  # 解析引用
    recursive: true

# 错误处理策略
error_handling:
  on_missing_reference: "placeholder"  # 缺失引用时显示占位符
  on_circular_reference: "error"       # 循环引用时停止处理
  max_recursion_depth: 5               # 防止过深嵌套
  placeholder_template: |
    <!-- 错误: 引用缺失 -->
    **[内容缺失: {source_uri}]**
    <!-- 请检查引用路径是否正确 -->

# 输出配置
output:
  format: single-file
  filename: "composed-document.md"
  header: |
    # 动态组合文档
    
    > 本文档由 elfi Recipe 系统自动生成
    > 生成时间: {timestamp}
    
```

## 预期输出示例

当Recipe正常执行时，生成的 `composed-document.md` 应该包含：

```markdown
# 动态组合文档

> 本文档由 elfi Recipe 系统自动生成
> 生成时间: 2024-01-15 14:30:00

## 项目介绍
[来自 intro 区块的内容]

## 共享工具
以下是来自 component.elf 的实用工具：

```python
def calculate_sum(a, b):
    """Calculate the sum of two numbers."""
    return a + b

def format_message(name):
    """Format a greeting message."""
    return f"Hello from {name}!"
```

## 使用说明
[来自 usage-guide 区块的内容]
```

这样的输出证明了 Recipe 系统成功地将其他文档的内容动态组合到了主文档中，实现了真正的“文档即App”效果。

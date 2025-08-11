---
id: block-A
type: markdown
---
# 分析帕尔默群岛企鹅数据

本文档演示了对帕尔默企鹅数据集的分析。我们将加载数据，执行计算，并创建图表。

---
id: block-B
type: markdown
metadata:
  parent: block-A
---
## 1. 数据加载

首先，让我们加载必要的库和数据集。

---
id: block-C
type: code
metadata:
  parent: block-B
  language: python
---
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

# 加载企鹅数据集
penguins = sns.load_dataset("penguins")

# 显示前几行数据
penguins.head()

---
id: block-D
type: markdown
metadata:
  parent: block-A
---
## 2. 数据可视化

现在，让我们创建一个散点图来可视化鳍状肢长度和喙长度之间的关系，并按物种进行颜色区分。

---
id: block-E
type: code
metadata:
  parent: block-D
  language: python
  interactive: true
---
# 创建散点图
plt.figure(figsize=(10, 6))
sns.scatterplot(
    data=penguins,
    x="flipper_length_mm",
    y="bill_length_mm",
    hue="species",
    style="species",
    s=100
)
plt.title("Flipper Length vs. Bill Length by Species")
plt.xlabel("Flipper Length (mm)")
plt.ylabel("Bill Length (mm)")
plt.grid(True)
plt.show()

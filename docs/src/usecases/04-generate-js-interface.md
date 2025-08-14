# TypeScript 接口生成：从 .elf 到代码的完整工作流

本用例展示了如何使用 ELFI 作为通用编程库进行代码生成，体现了 ELFI **结构与语义分离** 的核心设计理念。

## 核心概念

### 1. 用户定义的类型系统

在这个示例中，`api_interface` 类型**完全由用户项目定义**：

```yaml
# .elf 文件中的类型声明
type: api_interface  # 这是用户自定义的类型，非 ELFI 内置
```

**重要说明**：
- ELFI 系统**不预设** `api_interface` 的任何语义或处理逻辑
- 类型的含义、验证规则、转换方式都由用户项目定义
- ELFI 只负责解析、存储、传递和基于 Recipe 转换这些内容

### 2. 代码生成 vs Extension 系统

这是两个不同层次的功能：

| 功能类型 | 本示例（代码生成） | Extension 系统 |
|----------|------------------|---------------|
| **实现方式** | .elf 文件 + Recipe 系统 | `elfi install` + 插件 |
| **适用场景** | 基础的代码生成转换 | 扩展 ELFI 核心功能 |
| **安装需求** | 无需额外安装 | 需要安装 Extension |
| **灵活性** | 配置驱动的模板转换 | 可编程的功能扩展 |

## 工作流演示

### 第 1 步：定义结构化的 API 接口

```elf
---
id: a1b2c3d4-5e6f-7890-1234-56789abcdef0  
type: api_interface  # 用户自定义类型
name: user-service-api
attributes:
  description: "用户服务的核心 API 接口定义"
  version: "v1.0.0"
  namespace: "UserService"
---
interface UserServiceApi {
  /**
   * 获取用户基本信息
   */
  getUser(userId: string): Promise<User | null>;
  
  /**
   * 创建新用户
   */
  createUser(userData: CreateUserRequest): Promise<User>;
  
  // ... 更多接口定义
}
```

### 第 2 步：配置 Recipe 转换规则

```yaml
# Recipe 块定义如何转换用户的 api_interface 类型
name: "typescript-interface-generator"
version: "1.0.0"

selector:
  types: ["api_interface"]  # 选择用户定义的类型

transform:
  - type: "content_parser"
    action: "extract_typescript_interfaces"
  - type: "code_formatter"  
    action: "format_typescript"
  - type: "module_generator"
    action: "create_typescript_module"

output:
  format: "typescript"
  filename: "{{namespace | lower}}-api.ts"
```

### 第 3 步：执行代码生成

```bash
# 使用 ELFI CLI 生成 TypeScript 代码
elfi export 04-generate-js-interface.elf typescript-generator ./output/

# 输出: ./output/userservice-api.ts
```

### 第 4 步：生成的 TypeScript 代码

```typescript
/**
 * 用户服务的核心 API 接口定义
 * Version: v1.0.0
 * Generated from ELFI document
 * 
 * 注意：这些类型定义完全由用户在 .elf 文档中定义
 * ELFI 系统只负责结构化存储和代码生成转换
 */

interface UserServiceApi {
  /**
   * 获取用户基本信息
   * @param userId 用户唯一标识符
   * @returns 用户信息对象或null
   */
  getUser(userId: string): Promise<User | null>;
  
  /**
   * 创建新用户
   * @param userData 用户注册数据
   * @returns 创建的用户信息
   */
  createUser(userData: CreateUserRequest): Promise<User>;
  
  // ... 其他接口方法
}

interface User {
  id: string;
  username: string;
  email: string;
  displayName: string;
  avatar?: string;
  createdAt: Date;
  updatedAt: Date;
  profile: UserProfile;
}

// ... 其他类型定义

// 导出所有类型
export type {
  UserServiceApi,
  User,
  UserProfile,
  CreateUserRequest,
  UpdateUserRequest,
  SearchUsersRequest,
  SearchUsersResponse,
  ApiError,
  ValidationError,
  NotFoundError,
  ConflictError,
};
```

## 应用集成

### 在项目中使用生成的类型

```typescript
// 导入生成的类型
import type { 
  UserServiceApi, 
  User, 
  CreateUserRequest 
} from './generated/userservice-api';

// 实现接口
class UserServiceImpl implements UserServiceApi {
  async getUser(userId: string): Promise<User | null> {
    const response = await fetch(`/api/users/${userId}`);
    if (response.status === 404) {
      return null;
    }
    return response.json();
  }
  
  async createUser(userData: CreateUserRequest): Promise<User> {
    const response = await fetch('/api/users', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(userData),
    });
    return response.json();
  }
  
  // ... 其他方法实现
}

// 在应用中使用
const userService = new UserServiceImpl();
const user = await userService.getUser('123');
```

### 集成到构建流程

```json
{
  "scripts": {
    "generate-types": "elfi export docs/api-definitions/ typescript-generator src/types/generated/",
    "prebuild": "npm run generate-types",
    "build": "tsc && vite build"
  },
  "dependencies": {
    "elfi-cli": "^1.0.0"
  }
}
```

## 扩展能力

### 1. 自定义更多接口类型

用户可以根据项目需要定义各种类型：

```elf
---
type: grpc_service  # 用户定义的 gRPC 服务类型
name: payment-service
---
service PaymentService {
  rpc ProcessPayment(PaymentRequest) returns (PaymentResponse);
  rpc RefundPayment(RefundRequest) returns (RefundResponse);
}

---
type: graphql_schema  # 用户定义的 GraphQL 类型
name: user-schema
---
type User {
  id: ID!
  name: String!
  email: String!
}

type Query {
  user(id: ID!): User
  users: [User!]!
}
```

### 2. 创建多种输出格式

通过不同的 Recipe，可以从同一个 API 定义生成多种格式：

```bash
# 生成 TypeScript 接口
elfi export api-spec.elf typescript-generator ./ts-types/

# 生成 OpenAPI 规范
elfi export api-spec.elf openapi-generator ./docs/api-spec.yaml

# 生成 Python 类型提示
elfi export api-spec.elf python-types-generator ./python-types/

# 生成 Go 接口
elfi export api-spec.elf golang-interfaces-generator ./go-types/
```

### 3. 与 Extension 系统结合

对于更复杂的需求，可以通过 Extension 系统：

```bash
# 安装支持 Protocol Buffers 的 Extension
elfi install protobuf-extension

# 安装支持数据库 Schema 生成的 Extension  
elfi install database-schema-extension

# 使用 Extension 提供的高级功能
elfi generate-proto api-spec.elf --output ./proto/
elfi migrate-schema api-spec.elf --database postgres --output ./migrations/
```

## 设计理念体现

### 结构与语义分离

1. **ELFI 职责**：
   - 解析 `.elf` 文件结构
   - 存储用户定义的内容
   - 执行 Recipe 转换
   - 提供基础的模板和格式化能力

2. **用户职责**：
   - 定义所有类型的语义（`api_interface`, `grpc_service` 等）
   - 创建适合项目的 Recipe 转换规则
   - 决定代码生成的格式和结构

### 通用编程库特性

- **类型无关**：支持任何用户定义的类型和内容格式
- **转换灵活**：通过 Recipe 系统支持任意的内容转换
- **工具集成**：可以集成到任何构建流程和开发工作流
- **生态扩展**：通过 Extension 系统支持社区贡献

这个示例展示了 ELFI 作为通用编程库的强大能力：不预设任何业务逻辑，但提供强大的结构化文档和转换能力，让用户可以根据需要构建任何类型的代码生成工作流。
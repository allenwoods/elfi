# ELFI 开发环境配置脚本 (Windows PowerShell)
# 为 Windows 开发者提供的自动化配置

param(
    [switch]$SkipRust,
    [switch]$Offline
)

# 颜色定义
$Colors = @{
    Red    = "Red"
    Green  = "Green" 
    Yellow = "Yellow"
    Blue   = "Blue"
    Purple = "Magenta"
}

# 图标
$Icons = @{
    Check  = "✅"
    Cross  = "❌"
    Arrow  = "🔄"
    Rocket = "🚀"
}

function Write-ColorText {
    param([string]$Text, [string]$Color = "White")
    Write-Host $Text -ForegroundColor $Color
}

function Write-Header {
    Write-ColorText @"

╔════════════════════════════════════════╗
║       ELFI 开发环境配置脚本              ║
║   Event-sourcing Literate File        ║
║           Interpreter                  ║
║         (Windows PowerShell)           ║
╚════════════════════════════════════════╝

"@ -Color $Colors.Purple
}

function Test-CommandExists {
    param([string]$Command)
    try {
        Get-Command $Command -ErrorAction Stop | Out-Null
        return $true
    }
    catch {
        return $false
    }
}

function Install-Chocolatey {
    Write-ColorText "$($Icons.Arrow) 检查 Chocolatey..." -Color $Colors.Yellow
    
    if (Test-CommandExists "choco") {
        Write-ColorText "$($Icons.Check) Chocolatey 已安装" -Color $Colors.Green
        return
    }
    
    Write-ColorText "$($Icons.Arrow) 安装 Chocolatey..." -Color $Colors.Yellow
    
    Set-ExecutionPolicy Bypass -Scope Process -Force
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
    
    try {
        Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
        Write-ColorText "$($Icons.Check) Chocolatey 安装成功" -Color $Colors.Green
    }
    catch {
        Write-ColorText "$($Icons.Cross) Chocolatey 安装失败: $($_.Exception.Message)" -Color $Colors.Red
        Write-ColorText "请手动访问 https://chocolatey.org/install 安装 Chocolatey" -Color $Colors.Yellow
        exit 1
    }
}

function Install-Rust {
    if ($SkipRust) {
        Write-ColorText "跳过 Rust 安装" -Color $Colors.Yellow
        return
    }
    
    Write-ColorText "$($Icons.Arrow) 检查 Rust 安装..." -Color $Colors.Yellow
    
    if ((Test-CommandExists "rustc") -and (Test-CommandExists "cargo")) {
        $rustVersion = & rustc --version
        Write-ColorText "$($Icons.Check) Rust 已安装: $rustVersion" -Color $Colors.Green
        return
    }
    
    Write-ColorText "$($Icons.Arrow) 安装 Rust..." -Color $Colors.Yellow
    Write-ColorText "正在下载 rustup-init.exe..." -Color $Colors.Blue
    
    try {
        $rustupUrl = "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
        $rustupPath = "$env:TEMP\rustup-init.exe"
        
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath
        
        Write-ColorText "运行 Rust 安装程序..." -Color $Colors.Blue
        Start-Process -FilePath $rustupPath -ArgumentList "-y" -Wait
        
        # 刷新环境变量
        $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH","User")
        
        if ((Test-CommandExists "rustc") -and (Test-CommandExists "cargo")) {
            Write-ColorText "$($Icons.Check) Rust 安装成功" -Color $Colors.Green
        }
        else {
            throw "Rust 工具链未正确安装"
        }
    }
    catch {
        Write-ColorText "$($Icons.Cross) Rust 安装失败: $($_.Exception.Message)" -Color $Colors.Red
        Write-ColorText "请手动访问 https://rustup.rs/ 下载并安装 Rust" -Color $Colors.Yellow
        exit 1
    }
}

function Install-SystemDeps {
    Write-ColorText "$($Icons.Arrow) 安装系统依赖..." -Color $Colors.Yellow
    
    $deps = @("git", "vscode")
    
    foreach ($dep in $deps) {
        if (Test-CommandExists $dep) {
            Write-ColorText "$($Icons.Check) $dep 已安装" -Color $Colors.Green
        }
        else {
            Write-ColorText "$($Icons.Arrow) 通过 Chocolatey 安装 $dep..." -Color $Colors.Yellow
            try {
                & choco install $dep -y
                Write-ColorText "$($Icons.Check) $dep 安装成功" -Color $Colors.Green
            }
            catch {
                Write-ColorText "$($Icons.Cross) $dep 安装失败" -Color $Colors.Red
            }
        }
    }
}

function Install-RustTools {
    Write-ColorText "$($Icons.Arrow) 安装 Rust 开发工具..." -Color $Colors.Yellow
    
    $tools = @("just", "mdbook", "mdbook-mermaid")
    
    foreach ($tool in $tools) {
        if (Test-CommandExists $tool) {
            Write-ColorText "$($Icons.Check) $tool 已安装" -Color $Colors.Green
        }
        else {
            Write-ColorText "$($Icons.Arrow) 安装 $tool..." -Color $Colors.Yellow
            try {
                & cargo install $tool
                Write-ColorText "$($Icons.Check) $tool 安装成功" -Color $Colors.Green
            }
            catch {
                Write-ColorText "$($Icons.Cross) $tool 安装失败" -Color $Colors.Red
            }
        }
    }
}

function Setup-DocsEnvironment {
    Write-ColorText "$($Icons.Arrow) 配置文档环境..." -Color $Colors.Yellow
    
    if (-not (Test-Path "docs")) {
        Write-ColorText "$($Icons.Cross) 找不到 docs 目录，请确保在项目根目录运行此脚本" -Color $Colors.Red
        exit 1
    }
    
    Push-Location docs
    
    try {
        if (Test-Path "book.toml") {
            Write-ColorText "$($Icons.Arrow) 配置 mdbook-mermaid..." -Color $Colors.Yellow
            & mdbook-mermaid install .
            Write-ColorText "$($Icons.Check) mermaid 配置完成" -Color $Colors.Green
        }
        
        Write-ColorText "$($Icons.Arrow) 测试文档构建..." -Color $Colors.Yellow
        & mdbook build
        Write-ColorText "$($Icons.Check) 文档构建成功" -Color $Colors.Green
    }
    catch {
        Write-ColorText "$($Icons.Cross) 文档配置失败: $($_.Exception.Message)" -Color $Colors.Red
    }
    finally {
        Pop-Location
    }
}

function Setup-IDEConfig {
    Write-ColorText "$($Icons.Arrow) 创建 IDE 配置文件..." -Color $Colors.Yellow
    
    # 创建 .vscode 目录
    if (-not (Test-Path ".vscode")) {
        New-Item -ItemType Directory -Path ".vscode" | Out-Null
    }
    
    # VSCode 扩展推荐
    $extensionsJson = @{
        recommendations = @(
            "rust-lang.rust-analyzer",
            "tamasfe.even-better-toml",
            "yzhang.markdown-all-in-one",
            "bierner.markdown-mermaid",
            "skellock.just",
            "vadimcn.vscode-lldb"
        )
    } | ConvertTo-Json -Depth 3
    
    $extensionsJson | Out-File -FilePath ".vscode\extensions.json" -Encoding UTF8
    
    # VSCode 设置
    $settingsJson = @{
        "rust-analyzer.cargo.buildScripts.enable" = $true
        "rust-analyzer.checkOnSave.command" = "clippy"
        "editor.formatOnSave" = $true
        "editor.rulers" = @(100)
        "markdown.extension.toc.levels" = "2..6"
        "files.watcherExclude" = @{
            "**/target/**" = $true
            "**/docs/book/**" = $true
        }
    } | ConvertTo-Json -Depth 3
    
    $settingsJson | Out-File -FilePath ".vscode\settings.json" -Encoding UTF8
    
    Write-ColorText "$($Icons.Check) VSCode 配置文件已创建" -Color $Colors.Green
}

function Test-Installation {
    Write-ColorText "$($Icons.Arrow) 验证安装..." -Color $Colors.Yellow
    
    $tools = @("rustc", "cargo", "just", "mdbook", "mdbook-mermaid")
    $errors = 0
    
    foreach ($tool in $tools) {
        if (Test-CommandExists $tool) {
            try {
                $version = & $tool --version 2>$null | Select-Object -First 1
                Write-ColorText "$($Icons.Check) $tool`: $version" -Color $Colors.Green
            }
            catch {
                Write-ColorText "$($Icons.Check) $tool`: 已安装" -Color $Colors.Green
            }
        }
        else {
            Write-ColorText "$($Icons.Cross) $tool`: 未找到" -Color $Colors.Red
            $errors++
        }
    }
    
    return $errors -eq 0
}

function Show-NextSteps {
    Write-ColorText "$($Icons.Rocket) 配置完成！" -Color $Colors.Green
    Write-ColorText @"

📋 后续步骤:
   1. 重启 PowerShell 或重新加载环境变量
   2. 启动文档服务器: cd docs; just serve
   3. 运行测试: cargo test  
   4. 查看所有任务: just --list

🔗 有用的链接:
   • 📖 文档: http://localhost:3000
   • 🦀 Rust 学习: https://rustlings.cool/
   • 📚 mdBook 指南: https://rust-lang.github.io/mdBook/

Happy coding! 🎉
"@ -Color $Colors.Blue
}

# 主函数
function Main {
    Write-Header
    
    Write-ColorText "开始配置 Windows 开发环境..." -Color $Colors.Blue
    
    Install-Chocolatey
    Install-SystemDeps  
    Install-Rust
    Install-RustTools
    Setup-DocsEnvironment
    Setup-IDEConfig
    
    if (Test-Installation) {
        Show-NextSteps
    }
    else {
        Write-ColorText "$($Icons.Cross) 配置过程中出现错误，请检查上述输出" -Color $Colors.Red
        exit 1
    }
}

# 检查 PowerShell 版本
if ($PSVersionTable.PSVersion.Major -lt 5) {
    Write-ColorText "需要 PowerShell 5.0 或更高版本" -Color $Colors.Red
    exit 1
}

# 检查是否以管理员身份运行（Chocolatey 需要）
if (-not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-ColorText "⚠️  建议以管理员身份运行此脚本以安装系统依赖" -Color $Colors.Yellow
    $continue = Read-Host "是否继续？(y/N)"
    if ($continue -ne "y") {
        exit 1
    }
}

# 运行主函数
Main
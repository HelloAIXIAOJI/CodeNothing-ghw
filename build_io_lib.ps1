# PowerShell脚本用于编译library_io库并复制到正确的位置

# 进入library_io目录
cd .\library_io

# 编译库（release模式）
Write-Host "正在编译library_io..."
cargo build --release

# 创建目标目录（如果不存在）
$targetDir = "..\target\debug\library"
if (-not (Test-Path $targetDir)) {
    Write-Host "创建目录: $targetDir"
    New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
}

# 复制DLL文件
$sourceFile = ".\target\release\io.dll"
$targetFile = "$targetDir\io.dll"

Write-Host "复制文件: $sourceFile -> $targetFile"
Copy-Item -Path $sourceFile -Destination $targetFile -Force

# 返回原目录
cd ..

Write-Host "完成！io.dll已更新" 
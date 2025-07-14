@echo off
REM 批处理脚本用于编译library_io库并复制到正确的位置

REM 进入library_io目录
cd library_io

REM 编译库（release模式）
echo Compiling library_io...
cargo build --release

REM 创建目标目录（如果不存在）
if not exist "..\target\debug\library" (
    echo Creating directory: ..\target\debug\library
    mkdir "..\target\debug\library"
)

REM 复制DLL文件
echo Copying file: .\target\release\io.dll -^> ..\target\debug\library\io.dll
copy /Y ".\target\release\io.dll" "..\target\debug\library\io.dll"

REM 返回原目录
cd ..

echo Done! io.dll has been updated 
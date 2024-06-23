set WORKSPACE=..

set LUBAN_DLL=%WORKSPACE%\tools\luban\src\Luban\bin\Release\net8.0\Luban.dll
set CONF_ROOT=%WORKSPACE%\datatables
set OUTPUT_DATA_DIR=%WORKSPACE%\assets\datatables\
set CODE_GEN_DIR=%WORKSPACE%\crates\datatables\gen

dotnet %LUBAN_DLL% ^
    -t all ^
    -c rust-bin ^
    -d bin^
    --conf %CONF_ROOT%\luban.conf ^
    --customTemplateDir %CONF_ROOT%\templates ^
    -x outputCodeDir=%CODE_GEN_DIR% ^
    -x outputDataDir=%OUTPUT_DATA_DIR%

pause
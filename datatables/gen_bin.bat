set WORKSPACE=..
set LUBAN_DLL=%WORKSPACE%\tools\luban\src\Luban\bin\Release\net8.0\Luban.dll
set CONF_ROOT=.

dotnet %LUBAN_DLL% ^
    -t all ^
    -c rust-bin ^
    -d bin ^
    --conf %CONF_ROOT%\luban.conf ^
    --customTemplateDir %CONF_ROOT%\templates ^
    -x outputCodeDir=gen_bin ^
    -x outputDataDir=output\bin

pause
set WORKSPACE=..\..

set LUBAN_DLL=%WORKSPACE%\tools\luban\src\Luban\bin\Release\net8.0\Luban.dll
set CONF_ROOT=%WORKSPACE%\datatables

dotnet %LUBAN_DLL% ^
    -t all ^
    -c rust-json ^
    -d json  ^
    --conf %CONF_ROOT%\luban.conf ^
    --customTemplateDir %CONF_ROOT%\templates ^
    -x outputCodeDir=gen ^
    -x outputDataDir=%WORKSPACE%\assets\datatables\

pause
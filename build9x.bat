set target=i586-rust9x-windows-msvc
set editbin=C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.37.32822\bin\Hostx64\x86\editbin.exe

cargo +rust9x build %* --target %target% || exit /b %ERRORLEVEL%

for /F "tokens=*" %%e in ('dir /B /S "target\%target%\*.exe" 2^>NUL') do (
	"%editbin%" "%%e" /SUBSYSTEM:WINDOWS,4.0 /RELEASE >NUL
)

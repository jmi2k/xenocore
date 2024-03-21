set target=i586-rust9x-windows-msvc
set destination=\\CORTANA\Nexus

for /F "tokens=*" %%e in ('dir /B /S "target\%target%\*.exe" 2^>NUL') do (
	copy "%%e" "%destination%"
)

@echo off
REM Mock LLM script for integration testing on Windows
REM Captures invocation details to a JSON file for test verification
REM
REM Environment variables:
REM   MOCK_LLM_OUTPUT_DIR - Directory to write invocation files (default: %TEMP%\operator-test)
REM
REM Output: Creates invocation-{timestamp}.json with all captured data

setlocal enabledelayedexpansion

REM Set output directory
if "%MOCK_LLM_OUTPUT_DIR%"=="" (
    set "OUTPUT_DIR=%TEMP%\operator-test"
) else (
    set "OUTPUT_DIR=%MOCK_LLM_OUTPUT_DIR%"
)

REM Create output directory if it doesn't exist
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"

REM Generate unique invocation ID using time
for /f "tokens=1-4 delims=:. " %%a in ("%TIME%") do (
    set "INVOCATION_ID=%DATE:~-4%%DATE:~4,2%%DATE:~7,2%%%a%%b%%c%%d"
)
set "INVOCATION_FILE=%OUTPUT_DIR%\invocation-%INVOCATION_ID%.json"

REM Capture all arguments
set "ALL_ARGS=%*"
set "SESSION_ID="
set "MODEL="
set "PROMPT_FILE="

REM Parse arguments
:parse_args
if "%~1"=="" goto done_parsing
if "%~1"=="--session-id" (
    set "SESSION_ID=%~2"
    shift
    shift
    goto parse_args
)
if "%~1"=="--model" (
    set "MODEL=%~2"
    shift
    shift
    goto parse_args
)
if "%~1"=="--print-prompt-path" (
    set "PROMPT_FILE=%~2"
    shift
    shift
    goto parse_args
)
shift
goto parse_args
:done_parsing

REM Get current directory
set "CWD=%CD%"

REM Get timestamp
for /f "tokens=1-3 delims=/ " %%a in ('date /t') do set "DATESTAMP=%%c-%%a-%%b"
for /f "tokens=1-2 delims=: " %%a in ('time /t') do set "TIMESTAMP=%%a:%%b:00"

REM Write JSON output (simplified format due to batch limitations)
echo { > "%INVOCATION_FILE%"
echo     "timestamp": "%DATESTAMP%T%TIMESTAMP%Z", >> "%INVOCATION_FILE%"
echo     "invocation_id": "%INVOCATION_ID%", >> "%INVOCATION_FILE%"
echo     "command": "%~0", >> "%INVOCATION_FILE%"
echo     "args_raw": "%ALL_ARGS%", >> "%INVOCATION_FILE%"
echo     "session_id": "%SESSION_ID%", >> "%INVOCATION_FILE%"
echo     "model": "%MODEL%", >> "%INVOCATION_FILE%"
echo     "prompt_file": "%PROMPT_FILE%", >> "%INVOCATION_FILE%"
echo     "cwd": "%CWD:\=\\%" >> "%INVOCATION_FILE%"
echo } >> "%INVOCATION_FILE%"

REM Log for debugging
echo Mock LLM invoked
echo   Session ID: %SESSION_ID%
echo   Model: %MODEL%
echo   Prompt file: %PROMPT_FILE%
echo   Output: %INVOCATION_FILE%

REM Simulate a brief run and exit cleanly
timeout /t 1 /nobreak > nul 2>&1
exit /b 0

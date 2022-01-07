:: Batch file to build the following program variants:
::
:: - Windows
:: - Windows with all features (TLS and plugins)
:: - Linux
:: - Linux with all features (TLS and plugins)
:: - PHP plugin for Windows
:: - PHP plugin for Linux
::
:: Requires Rust to be installed both normally and in WSL.

@echo off

echo Setting up Humphrey build...

echo Building for Windows...
cargo build --release -q
robocopy target/release dist humphrey.exe > nul
cd dist
rename humphrey.exe humphrey_windows.exe
cd ..

echo Building for Linux..
wsl $HOME/.cargo/bin/cargo build --release -q
robocopy target/release dist humphrey > nul
cd dist
rename humphrey humphrey_linux
cd ..

echo Building for Windows (with all features)...
cargo build --release -q --all-features
robocopy target/release dist humphrey.exe > nul
cd dist
rename humphrey.exe humphrey_windows_all_features.exe
cd ..

echo Building for Linux (with all features)...
wsl $HOME/.cargo/bin/cargo build --release -q --all-features
robocopy target/release dist humphrey > nul
cd dist
rename humphrey humphrey_linux_all_features
cd ..

echo Building PHP plugin for Windows...
cd plugins/php
cargo build --release -q
robocopy target/release ../../dist php.dll > nul
cd ../../dist
rename php.dll php_plugin_windows.dll

echo Building PHP plugin for Linux...
cd ../plugins/php
wsl $HOME/.cargo/bin/cargo build --release -q
robocopy target/release ../../dist libphp.so > nul
cd ../../dist
rename libphp.so php_plugin_linux.so

cd ..

echo Build complete.
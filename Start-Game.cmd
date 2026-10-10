@echo off
cd /d "%~dp0"
if not defined CORDON_ASSETS set "CORDON_ASSETS=D:\GameAssets\cordon-royale\web"
cargo run --release

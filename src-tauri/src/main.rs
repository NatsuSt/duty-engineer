// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use env_logger;

fn main() {
    env_logger::init();
    duty_engineer_lib::run()
}

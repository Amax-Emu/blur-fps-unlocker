use actor_rotator::set_actor_rotator_hook;
use log::{debug, info, LevelFilter};
use physics_update::{physics_values_remove_protection, set_physics_update_hook};
use simplelog::{
    ColorChoice, CombinedLogger, Config, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};
use ui_messages_hook::set_message_broadcast_hook;

use std::{ffi::c_void, thread, time::Duration};

use windows::{
    core::PCSTR,
    Win32::System::Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS},
    Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    Win32::{Foundation::HMODULE, System::LibraryLoader::GetModuleHandleA},
};

mod actor_rotator;
mod physics_update;
mod ui_messages_hook;

pub static EXE_BASE_ADDR: isize = 0x00400000;

#[no_mangle]
#[allow(non_snake_case)]
extern "system" fn DllMain(
    dll_module: windows::Win32::Foundation::HMODULE,
    call_reason: u32,
    _reserved: *mut std::ffi::c_void,
) -> i32 {
    match call_reason {
        DLL_PROCESS_ATTACH => init(dll_module),
        DLL_PROCESS_DETACH => free(dll_module),
        _ => (),
    }
    true.into()
}

pub fn init(module: HMODULE) {
    init_logs();

    info!("Hi from: {module:X?}");

    info!("Installing hooks");
    set_actor_rotator_hook();

    physics_values_remove_protection();
    set_physics_update_hook();
    
    set_message_broadcast_hook();
    info!("Done!");
}

pub fn free(module: HMODULE) {
    log::info!("Bye from: {module:X?}");
}

fn init_logs() {
    let cfg = ConfigBuilder::new()
        .set_time_offset_to_local()
        .unwrap()
        .build();
    let log_file = blur_plugins_core::create_log_file("blur_fps.log").unwrap();
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Trace,
            cfg,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(LevelFilter::Trace, Config::default(), log_file),
    ])
    .unwrap();
    log_panics::init();
}

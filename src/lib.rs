use log::{debug, LevelFilter};
use simplelog::{
    ColorChoice, CombinedLogger, Config, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};
use std::{ffi::c_void, thread, time::Duration};

use windows::{
    core::PCSTR,
    Win32::System::Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS},
    Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    Win32::{Foundation::HMODULE, System::LibraryLoader::GetModuleHandleA},
};

use crate::physics_fixer::{
    dynamic_physics_update, fix_carousel, fix_powerups_rotation, get_p_instance,
};

mod physics_fixer;

pub static EXE_BASE_ADDR: i32 = 0x00400000;

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

    log::info!("Hi from: {module:X?}");

    debug!("Disabling memory protection");

    const TIMESTEP_PTR: i32 = 0x00E648B8;
    const TIMESTEP_SIZE_PTR: i32 = 0x00E65A2C;
    const MAX_PHYSICS_UPDATE_PTR: i32 = 0x0111C6A4;
    const POWERUPS_ROTATION_SPEED: i32 = 0x011BF428;

    const INS_CALL_LEN: isize = 4;

    // Windows will be angry if we write to protected memory!
    let src_flags = &mut PAGE_PROTECTION_FLAGS::default();
    let tmp_flags = PAGE_EXECUTE_READWRITE;

    let ptr_base: *mut std::ffi::c_void =
        unsafe { GetModuleHandleA(PCSTR::null()) }.unwrap().0 as _;

    let ingame_timestep =
        ptr_base.wrapping_byte_offset((TIMESTEP_PTR - EXE_BASE_ADDR) as isize) as *mut f32;
    let ingame_timestep_size =
        ptr_base.wrapping_byte_offset((TIMESTEP_SIZE_PTR - EXE_BASE_ADDR) as isize) as *mut f32;
    let ingame_max_physics_updates = ptr_base
        .wrapping_byte_offset((MAX_PHYSICS_UPDATE_PTR - EXE_BASE_ADDR) as isize)
        as *mut i32;
    let ingame_bonus_rotation_speed = ptr_base
        .wrapping_byte_offset((POWERUPS_ROTATION_SPEED - EXE_BASE_ADDR) as isize)
        as *mut f32;

    unsafe {
        VirtualProtect(
            ingame_timestep as _,
            INS_CALL_LEN as usize,
            tmp_flags,
            src_flags,
        )
        .unwrap()
    };

    unsafe {
        VirtualProtect(
            ingame_timestep_size as _,
            INS_CALL_LEN as usize,
            tmp_flags,
            src_flags,
        )
        .unwrap()
    };

    unsafe {
        VirtualProtect(
            ingame_max_physics_updates as _,
            INS_CALL_LEN as usize,
            tmp_flags,
            src_flags,
        )
        .unwrap()
    };

    unsafe {
        VirtualProtect(
            ingame_bonus_rotation_speed as _,
            INS_CALL_LEN as usize,
            tmp_flags,
            src_flags,
        )
        .unwrap()
    };

    log::debug!("Starting update!");

    fix_carousel();

    thread::spawn(|| loop {
        match get_p_instance() {
            Some(p_instance) => {
                dynamic_physics_update(p_instance);
                fix_powerups_rotation(p_instance);
            }
            None => todo!(),
        }

        thread::sleep(Duration::from_secs(1));
    });

    let _ptr_base: *mut c_void = unsafe { GetModuleHandleA(PCSTR::null()) }.unwrap().0 as _;
}

pub fn free(module: HMODULE) {
    log::info!("Bye from: {module:X?}");
}

// pub fn protection_disabler(target_addr: isize) {

// }

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

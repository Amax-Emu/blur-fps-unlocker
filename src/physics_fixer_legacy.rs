use log::debug;
use windows::{
    core::PCSTR,
    Win32::System::{
        LibraryLoader::GetModuleHandleA,
        Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS},
    },
};

use crate::EXE_BASE_ADDR;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CApplicationInstance {
    pub unk1_ptr: i32,
    pub unk2: f32,
    pub unk3: i32,
    pub unk4: f32,
    pub unk5: f32,
    pub unk6_phys_updates: f32,
    pub time_size_last_update: f32,
    pub unk7_timer: f32,
    pub fps1: f32,
}

pub fn get_p_instance() -> Option<CApplicationInstance> {
    //This is an object, called Framework::C_Application::s_CApplicationInstance, which is responsible for a lot of unknown, but very important stuff in whole game simulation
    //It contain timers, fps and many, many more critical cariables
    //As usual I just use it for something minor, like fps

    const CAPPLICATION_INSTANCE_FPS_PRT: *const i32 = 0x011aa04c as _;

    let c_application_instance_ptr =
        unsafe { *CAPPLICATION_INSTANCE_FPS_PRT as *const CApplicationInstance };

    if c_application_instance_ptr.is_null() {
        return None;
    }

    let p_instance = unsafe { *c_application_instance_ptr };
    return Some(p_instance);
}

pub fn dynamic_physics_update(p_instance: CApplicationInstance) {
    //Physics simulation in Blur consists of 2(3) values
    //Timestep - number of steps per second. Should be fps * 2, 60 by default. At least that's the value on ps3, which is 30 fps capped
    //Timestep size - how much time needs to pass to do a physics simulation.
    //Max physics update - limits how many times we do the physics update in 1 timestep. I'm still not 100% sure it needs to be increased from default value of 8. But let's do change it for now

    let ptr_base: *mut std::ffi::c_void =
        unsafe { GetModuleHandleA(PCSTR::null()) }.unwrap().0 as _;

    let timestep: f32 = (p_instance.fps1 * 2.0).round();
    let timestep_size: f32 = 1.0 / timestep;
    let physics_updates = (0.07 * timestep).round() as u32;

    const TIMESTEP_PTR: i32 = 0x00E648B8;
    const TIMESTEP_SIZE_PTR: i32 = 0x00E65A2C;
    const MAX_PHYSICS_UPDATE_PTR: i32 = 0x0111C6A4;

    let ingame_timestep =
        ptr_base.wrapping_byte_offset((TIMESTEP_PTR - EXE_BASE_ADDR) as isize) as *mut f32;
    let ingame_timestep_size =
        ptr_base.wrapping_byte_offset((TIMESTEP_SIZE_PTR - EXE_BASE_ADDR) as isize) as *mut f32;
    let ingame_max_physics_updates = ptr_base
        .wrapping_byte_offset((MAX_PHYSICS_UPDATE_PTR - EXE_BASE_ADDR) as isize)
        as *mut u32;

    unsafe {
        debug!(
            "Timestep: {}, size: {}, physics updates: {}",
            *ingame_timestep, *ingame_timestep_size, *ingame_max_physics_updates
        );
    }

    unsafe {
        ingame_timestep.write(timestep);
        ingame_timestep_size.write(timestep_size);
        ingame_max_physics_updates.write(physics_updates);
    }
}

pub fn fix_carousel() {
    //Somewhere there probably a UI timescale variable, which I haven't find yet. So for now I'll just apply a pure cosmetic fix for the menu, which is still rather twitchy

    const CAROUSEL_ROTATION_RATE: i32 = 0x01106438;

    const INS_CALL_LEN: isize = 4;

    // Windows will be angry if we write to protected memory!
    let src_flags = &mut PAGE_PROTECTION_FLAGS::default();
    let tmp_flags = PAGE_EXECUTE_READWRITE;

    unsafe {
        VirtualProtect(
            CAROUSEL_ROTATION_RATE as _,
            INS_CALL_LEN as usize,
            tmp_flags,
            src_flags,
        )
        .unwrap()
    };

    let ptr_base: *mut std::ffi::c_void =
        unsafe { GetModuleHandleA(PCSTR::null()) }.unwrap().0 as _;

    let carousel_rotation_rate_ptr = ptr_base
        .wrapping_byte_offset((CAROUSEL_ROTATION_RATE - EXE_BASE_ADDR) as isize)
        as *mut f32;

    unsafe { carousel_rotation_rate_ptr.write(300.0) }
}

pub fn fix_powerups_rotation(p_instance: CApplicationInstance) {
    //Technically, there should be a particular variable, that controls how ofter we updates stuff like sprites and some other minor physics/time related stuff
    //But since I haven't found it yet I'll do it a dirty way
    //Power-ups rotation is a hardcoded variable in some lua part of the code, that is copied by Power-ups object itself.
    //So in order to keep rotation rate on good level we need to update it constantly.
    //Not a great solution, but it is, for now

    let ptr_base: *mut std::ffi::c_void =
        unsafe { GetModuleHandleA(PCSTR::null()) }.unwrap().0 as _;

    let rotation = 3.0 / p_instance.fps1;

    debug!("New rotation speed {:?}", rotation);

    const POWERUPS_ROTATION_SPEED: i32 = 0x011BF428;

    let ingame_bonus_rotation_speed = ptr_base
        .wrapping_byte_offset((POWERUPS_ROTATION_SPEED - EXE_BASE_ADDR) as isize)
        as *mut f32;

    unsafe {
        ingame_bonus_rotation_speed.write(rotation);
    }
}

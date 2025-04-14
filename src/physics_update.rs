use log::debug;
use retour::static_detour;
use std::ffi::c_void;
use windows::{
    core::PCSTR,
    Win32::System::{
        LibraryLoader::GetModuleHandleA,
        Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS},
    },
};

use crate::{fake_speed_update::FAKE_PHYSICS_STABILIZATION_RATE, EXE_BASE_ADDR};

//void __fastcall .Game::C_GameWorld::UpdatePhysics(int param_1)
static_detour! {
    static PhysicsUpdate: unsafe extern "fastcall" fn(*mut c_void);
}

pub fn set_physics_update_hook() {
    let ptr_base: *mut c_void = unsafe { GetModuleHandleA(PCSTR::null()) }.unwrap().0 as _;
    let fn_base = ptr_base.wrapping_byte_offset(0x0041adc0 - EXE_BASE_ADDR) as i32;

    unsafe {
        let original_call =
            std::mem::transmute::<usize, unsafe extern "fastcall" fn(*mut c_void)>(fn_base as _);
        PhysicsUpdate
            .initialize(original_call, fake_physics_update)
            .unwrap()
            .enable()
            .unwrap()
    };
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CApplicationInstance {
    pub unk1_ptr: isize,
    pub unk2: f32,
    pub unk3: isize,
    pub unk4: f32,
    pub unk5: f32,
    pub unk6_phys_updates: f32,
    pub time_size_last_update: f32,
    pub unk7_timer: f32,
    pub fps1: f32,
}

const TIMESTEP_PTR: isize = 0x00E648B8;
const TIMESTEP_SIZE_PTR: isize = 0x00E65A2C;
const MAX_PHYSICS_UPDATE_PTR: isize = 0x0111C6A4;


const INS_CALL_LEN: isize = 4;

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

fn fake_physics_update(_self: *mut c_void) {
    //Physics fix

    //Problem: physics updates are tied to fps
    //Core issue: physics is updated each frame, but with fps increase it's still calculated with 30 fps as a target
    //With this hook we recalculate correct physics timesteps and values based on fps before physics update is performed

    let p_instance = match get_p_instance() {
        Some(p_instance) => p_instance,
        None => {
            //Should never happen, but just in case
            debug!("Instance is null!");
            return unsafe { PhysicsUpdate.call(_self) };
        }
    };

    let ptr_base: *mut std::ffi::c_void =
        unsafe { GetModuleHandleA(PCSTR::null()) }.unwrap().0 as _;

    let timestep: f32 = (p_instance.fps1 * 2.0).round(); //in base ps3 code equal 60 with 30 fps target
    let timestep_size: f32 = 1.0 / timestep; //in base ps3 code equal 0.01666667
    let physics_updates = (0.07 * timestep).round() as u32; //in base ps3 code equal 4. It's effect is rather unknown, but I update it anyway

    //Horrible, replace with direct ptrs for optimization
    let ingame_timestep = ptr_base.wrapping_byte_offset(TIMESTEP_PTR - EXE_BASE_ADDR) as *mut f32;
    let ingame_timestep_size =
        ptr_base.wrapping_byte_offset(TIMESTEP_SIZE_PTR - EXE_BASE_ADDR) as *mut f32;
    let ingame_max_physics_updates =
        ptr_base.wrapping_byte_offset(MAX_PHYSICS_UPDATE_PTR - EXE_BASE_ADDR) as *mut u32;

    unsafe {
        ingame_timestep.write(timestep);
        ingame_timestep_size.write(timestep_size);
        ingame_max_physics_updates.write(physics_updates); //still not entirely sure this one is required
    }

    #[cfg(debug_assertions)]
    unsafe {
        debug!(
            "Timestep: {}, size: {}, physics updates: {}",
            *ingame_timestep, *ingame_timestep_size, *ingame_max_physics_updates
        );
    }

    return unsafe { PhysicsUpdate.call(_self) };
}

pub fn physics_values_remove_protection() {
    // Windows will be angry if we write to protected memory!

    debug!("Disabling memory protection for physics values");

    let src_flags = &mut PAGE_PROTECTION_FLAGS::default();
    let tmp_flags = PAGE_EXECUTE_READWRITE;

    let ptr_base: *mut std::ffi::c_void =
        unsafe { GetModuleHandleA(PCSTR::null()) }.unwrap().0 as _;

    let ingame_timestep = ptr_base.wrapping_byte_offset(TIMESTEP_PTR - EXE_BASE_ADDR) as *mut f32;
    let ingame_timestep_size =
        ptr_base.wrapping_byte_offset(TIMESTEP_SIZE_PTR - EXE_BASE_ADDR) as *mut f32;
    let ingame_max_physics_updates =
        ptr_base.wrapping_byte_offset(MAX_PHYSICS_UPDATE_PTR - EXE_BASE_ADDR) as *mut i32;

    let fake_physics_stabilization_rate =
        ptr_base.wrapping_byte_offset(FAKE_PHYSICS_STABILIZATION_RATE - EXE_BASE_ADDR) as *mut i32;

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

    //Fake physics 

    unsafe {
        VirtualProtect(
            fake_physics_stabilization_rate as _,
            INS_CALL_LEN as usize,
            tmp_flags,
            src_flags,
        )
        .unwrap()
    };

}

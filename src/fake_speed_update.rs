use log::debug;
use retour::static_detour;
use std::ffi::c_void;
use windows::{core::PCSTR, Win32::System::LibraryLoader::GetModuleHandleA};

use crate::{ui_messages_hook::Message, EXE_BASE_ADDR};

pub const FAKE_PHYSICS_STABILIZATION_RATE: isize = 0x00e76504;

static_detour! {
    static FakeVehicleSpeedStateUpdate: unsafe extern "thiscall" fn(*mut c_void, *mut c_void, f32, *mut c_void);
}

pub fn set_fake_speed_hook() {
    let ptr_base: *mut c_void = unsafe { GetModuleHandleA(PCSTR::null()) }.unwrap().0 as _;
    let fn_base = ptr_base.wrapping_byte_offset(0x00488b20 - EXE_BASE_ADDR) as i32;

    unsafe {
        let original_call = std::mem::transmute::<
            usize,
            unsafe extern "thiscall" fn(*mut c_void, *mut c_void, f32, *mut c_void),
        >(fn_base as _);
        FakeVehicleSpeedStateUpdate
            .initialize(original_call, fake_fake_speed_update)
            .unwrap()
            .enable()
            .unwrap()
    };
}

fn fake_fake_speed_update(_self: *mut c_void, vehicle: *mut c_void, time_step: f32, unk1: *mut c_void ) {    
    
    debug!("_self: {:?} vehicle: {:?}, time_step: {:?}, unk1: {:?}", _self,vehicle,time_step, unk1 );

    let ptr_base: *mut std::ffi::c_void =
        unsafe { GetModuleHandleA(PCSTR::null()) }.unwrap().0 as _;

    let fake_physics_stabilization_rate =
    ptr_base.wrapping_byte_offset(FAKE_PHYSICS_STABILIZATION_RATE - EXE_BASE_ADDR) as *mut f32;

    let new_rate: f32 = 0.44694445 / (0.03333 / time_step);

    unsafe {
        fake_physics_stabilization_rate.write(new_rate);
    }

    unsafe { FakeVehicleSpeedStateUpdate.call(_self,  vehicle, time_step,unk1) }

    debug!("Called!")
}

use log::debug;
use retour::static_detour;
use std::ffi::c_void;
use windows::{core::PCSTR, Win32::System::LibraryLoader::GetModuleHandleA};

use crate::{ui_messages_hook::Message, EXE_BASE_ADDR};

pub const FAKE_PHYSICS_ACCELERATE_STABILIZE_RATE: isize = 0x0112e8e4; //from lua debug vars  def 10.0
pub const FAKE_PHYSICS_ACCELERATE_STABILIZE: isize = 0x0112e91c; //from lua debug vars def 5.0

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
    ptr_base.wrapping_byte_offset(FAKE_PHYSICS_ACCELERATE_STABILIZE_RATE - EXE_BASE_ADDR) as *mut f32;

    let fake_physics_stabilization =
    ptr_base.wrapping_byte_offset(FAKE_PHYSICS_ACCELERATE_STABILIZE - EXE_BASE_ADDR) as *mut f32;

    let new_rate: f32 = 10.0 * (time_step / 0.03333);
    let new_base: f32 = 5.0 * (time_step / 0.03333);
    unsafe {
        fake_physics_stabilization_rate.write(new_rate);
        fake_physics_stabilization.write(new_base);
    }

    unsafe { FakeVehicleSpeedStateUpdate.call(_self,  vehicle, time_step,unk1) }

    debug!("Called!")
}

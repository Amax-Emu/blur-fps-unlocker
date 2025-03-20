use log::debug;
use retour::static_detour;
use std::ffi::c_void;
use windows::{core::PCSTR, Win32::System::LibraryLoader::GetModuleHandleA};

static_detour! {
    static ActorRotator: unsafe extern "thiscall" fn(*mut c_void, *mut f32, *mut c_void) -> *mut c_void;
}

pub fn set_actor_rotator_hook() {
    let ptr_base: *mut c_void = unsafe { GetModuleHandleA(PCSTR::null()) }.unwrap().0 as _;
    let fn_base = ptr_base.wrapping_byte_offset(0x004e48e0 - 0x00400000) as i32;

    unsafe {
        let original_call = std::mem::transmute::<
            usize,
            unsafe extern "thiscall" fn(*mut c_void, *mut f32, *mut c_void) -> *mut c_void,
        >(fn_base as _);
        ActorRotator
            .initialize(original_call, fake_actor_rotator)
            .unwrap()
            .enable()
            .unwrap()
    };
}

fn fake_actor_rotator(_self: *mut c_void, rotation: *mut f32, unk1: *mut c_void) -> *mut c_void {
    //Rotating Actor Bug fix

    //Problem: rotating things are rotating based on fps
    //Core issue: rotating is calculated as "pi/2 - rotating_value" each frame
    //This code fix this issue with a time_step or "Frame Duration" value neatly located at offset 0x5c from rotating value.
    //Could be potentially disasterous for any other rotating things. In this case - use code in comments

    //This is defined by global debug Perks.PickupActor.RotationSpeed and set for each powerup actor on actor creation
    let amount_of_rotation_to_do = unsafe { *rotation };
    debug!("Amount : {}", amount_of_rotation_to_do);

    let frame_duration_ptr = rotation.clone().wrapping_byte_add(0x5C);

    let time_step = unsafe { *frame_duration_ptr }; //I'll call it time step, since it's called that in the code
    debug!("Time step in rotator: {}", time_step);

    unsafe { *rotation = amount_of_rotation_to_do * (60.0 / (1.0 / time_step)) }; //adjusting amount of rotation for our timestep

    // match get_p_instance() {
    //     Some(pInstance) => {
    //         let new_rotator_mul: f32 = 6.0 / pInstance.fps1;

    //         unsafe { *rotation = new_rotator_mul };

    //     },
    //     None => todo!(),
    // };

    #[cfg(debug_assertions)]
    {
        let time_step_1 = unsafe { *rotation };
        debug!("Time step in rotator: {}", time_step_1);
    }

    debug!("Rotator called");
    unsafe { ActorRotator.call(_self, rotation, unk1) }
}

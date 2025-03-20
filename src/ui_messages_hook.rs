use core::sync::atomic::{AtomicU64, Ordering};
use log::debug;
use retour::static_detour;
use std::ffi::c_void;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use windows::{core::PCSTR, Win32::System::LibraryLoader::GetModuleHandleA};

use crate::EXE_BASE_ADDR;

//void __thiscall
//UI::C_ButtonController::BroadcastMessage(C_ButtonController *this,C_MessageBase *pMessage,U32 m ask)
static_detour! {
    static UIBroadcastMessage: unsafe extern "thiscall" fn(*mut c_void,*mut Message,u32);
}

pub fn set_message_broadcast_hook() {
    let ptr_base: *mut c_void = unsafe { GetModuleHandleA(PCSTR::null()) }.unwrap().0 as _;
    let fn_base = ptr_base.wrapping_byte_offset(0x0074c9a0 - EXE_BASE_ADDR) as i32;

    unsafe {
        let original_call = std::mem::transmute::<
            usize,
            unsafe extern "thiscall" fn(*mut c_void, *mut Message, u32),
        >(fn_base as _);
        UIBroadcastMessage
            .initialize(original_call, fake_ui_broadcast_message)
            .unwrap()
            .enable()
            .unwrap()
    };
}

static LAST_UPDATE: AtomicU64 = AtomicU64::new(0);

pub fn set_value(val: u64) {
    LAST_UPDATE.store(val, Ordering::Relaxed)
}

pub fn get_value() -> u64 {
    LAST_UPDATE.load(Ordering::Relaxed)
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Message {
    message_type: u32,
    size: u32,
}

//0xA2193008 render UI layer
//0xA2193006 render animated objects?
//0xA2193005 render UI elements
//0xA2193004 screen resizing?

//0xa219300f - up
//0xa2193010 - down
//0xa2193011 - left
//0xa2193012 - right
//0x0A2193020 - backspace
static AUTO_FIRE_EVENTS: [u32; 5] = [0xa219300f, 0xa2193010, 0xa2193011, 0xa2193012, 0x0A2193020];

fn fake_ui_broadcast_message(_self: *mut c_void, message_ptr: *mut Message, mask: u32) {
    let message = unsafe { *message_ptr };

    #[cfg(debug_assertions)]
    if ![
        0xA2193004, 0xA2193005, 0xA2193006, 0xA2193008, 0xA2193035, 0xA2193036,
    ]
    .contains(&message.message_type)
    {
        debug!("message type :{:X}", message.message_type);
    }

    if !AUTO_FIRE_EVENTS.contains(&message.message_type) {
        return unsafe { UIBroadcastMessage.call(_self, message_ptr, mask) };
    };

    //This should be called only each 0.0166 seconds, or 0.033 for true expirience.
    //^ Didn't allign with actual tests

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards") //Unwrap default?
        .as_millis() as u64;

    let last = get_value();

    debug!("delta {}", now - last);

    if now - last >= 60 {
        //60ms feels the best, but we're dangerously in 50+ ms territory
        set_value(now);
        debug!("Brodcasting the message");
        unsafe { UIBroadcastMessage.call(_self, message_ptr, mask) }
    } else {
        debug!("Skipping broadcasting");
    }
}

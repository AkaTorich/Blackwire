//! Запас стека машины. Рекурсия Adder исполняется рекурсией Rust, поэтому кроме
//! лимита глубины (1000) нужна защита от настоящего переполнения стека потока.
//! На Windows нижняя граница стека узнаётся точно (`VirtualQuery`) — работает для
//! потока с любым размером стека; иначе действует фиксированный бюджет от точки входа.

/// Сколько байт оставляем встроенным функциям и хосту ниже текущей точки.
pub const STACK_MARGIN: usize = 384 * 1024;

/// Адрес текущей вершины стека (приблизительно).
#[inline(never)]
pub fn here() -> usize {
    let marker = 0u8;
    &marker as *const u8 as usize
}

/// Нижняя граница стека текущего потока, если её можно узнать.
pub fn bottom(here: usize) -> Option<usize> {
    #[cfg(windows)]
    {
        win::stack_bottom(here)
    }
    #[cfg(not(windows))]
    {
        let _ = here;
        None
    }
}

#[cfg(windows)]
mod win {
    use std::ffi::c_void;

    #[repr(C)]
    struct MemoryBasicInformation {
        base_address: *mut c_void,
        allocation_base: *mut c_void,
        allocation_protect: u32,
        #[cfg(target_pointer_width = "64")]
        partition_id: u16,
        region_size: usize,
        state: u32,
        protect: u32,
        kind: u32,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn VirtualQuery(address: *const c_void, buffer: *mut MemoryBasicInformation, length: usize) -> usize;
    }

    /// Начало области, зарезервированной под стек потока (самый нижний адрес).
    pub fn stack_bottom(here: usize) -> Option<usize> {
        let mut info = std::mem::MaybeUninit::<MemoryBasicInformation>::uninit();
        let written = unsafe {
            VirtualQuery(here as *const c_void, info.as_mut_ptr(), std::mem::size_of::<MemoryBasicInformation>())
        };
        if written == 0 {
            return None;
        }
        let info = unsafe { info.assume_init() };
        let base = info.allocation_base as usize;
        if base == 0 || base >= here {
            None
        } else {
            Some(base)
        }
    }
}

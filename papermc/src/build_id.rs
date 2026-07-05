//! Read the running DSO's GNU build id from its loaded ELF image.

use std::ffi::c_void;

/// The GNU build id (`.note.gnu.build-id`) of the DSO containing this function, as lowercase
/// hex, or `None` if the module has no build id note.
///
/// Parsed from the loaded image via `dl_iterate_phdr(3)`, so it identifies the code actually
/// executing. That can differ from the file on disk at the same path: `dlopen(3)` matches
/// already-loaded objects by path, so a mapping that outlives a reload cycle keeps running even
/// after the file underneath it has been replaced by a rebuild. Comparing this value against
/// `readelf -n <path>` answers "is the running code the code on disk".
pub fn running_build_id() -> Option<String> {
    let mut info: libc::Dl_info = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::dladdr(running_build_id as *const c_void, &mut info) };
    if rc == 0 || info.dli_fbase.is_null() {
        return None;
    }

    struct State {
        target_base: usize,
        build_id: Option<String>,
    }
    let mut state = State {
        target_base: info.dli_fbase as usize,
        build_id: None,
    };

    unsafe extern "C" fn callback(
        phdr_info: *mut libc::dl_phdr_info,
        _size: libc::size_t,
        data: *mut c_void,
    ) -> libc::c_int {
        let state = unsafe { &mut *(data as *mut State) };
        let phdr_info = unsafe { &*phdr_info };
        // dlpi_addr is the relocation base, which for a shared object equals dladdr's dli_fbase.
        if phdr_info.dlpi_addr as usize != state.target_base {
            return 0;
        }
        let phdrs = unsafe {
            std::slice::from_raw_parts(phdr_info.dlpi_phdr, phdr_info.dlpi_phnum as usize)
        };
        for phdr in phdrs {
            if phdr.p_type != libc::PT_NOTE {
                continue;
            }
            let notes = unsafe {
                std::slice::from_raw_parts(
                    (state.target_base + phdr.p_vaddr as usize) as *const u8,
                    phdr.p_memsz as usize,
                )
            };
            if let Some(id) = parse_build_id_note(notes) {
                state.build_id = Some(id);
                return 1;
            }
        }
        0
    }

    unsafe { libc::dl_iterate_phdr(Some(callback), (&raw mut state).cast::<c_void>()) };
    state.build_id
}

/// Scan one PT_NOTE segment for an `NT_GNU_BUILD_ID` note and render its payload as hex.
///
/// Note layout per elf(5): `Elf64_Nhdr` is three u32s (namesz, descsz, type) followed by the
/// name and descriptor, each padded to 4-byte alignment.
fn parse_build_id_note(mut notes: &[u8]) -> Option<String> {
    const NT_GNU_BUILD_ID: u32 = 3;
    while notes.len() >= 12 {
        let namesz = u32::from_ne_bytes(notes[0..4].try_into().ok()?) as usize;
        let descsz = u32::from_ne_bytes(notes[4..8].try_into().ok()?) as usize;
        let n_type = u32::from_ne_bytes(notes[8..12].try_into().ok()?);
        let name_end = 12usize.checked_add(namesz)?;
        let desc_start = name_end.next_multiple_of(4);
        let desc_end = desc_start.checked_add(descsz)?;
        if desc_end > notes.len() {
            return None;
        }
        if n_type == NT_GNU_BUILD_ID && namesz == 4 && notes.get(12..16) == Some(b"GNU\0") {
            let mut hex = String::with_capacity(descsz * 2);
            for byte in &notes[desc_start..desc_end] {
                hex.push_str(&format!("{byte:02x}"));
            }
            return Some(hex);
        }
        let next = desc_end.next_multiple_of(4);
        if next >= notes.len() {
            return None;
        }
        notes = &notes[next..];
    }
    None
}

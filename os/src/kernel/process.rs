//! 进程相关的内核功能

use super::*;
use xmas_elf::ElfFile;
use crate::fs::ROOT_INODE;
use crate::fs::INodeExt;

pub(super) fn sys_exit(code: usize) -> SyscallResult {
    println!(
        "thread {} exit with code {}",
        PROCESSOR.lock().current_thread().id,
        code
    );
    SyscallResult::Kill
}

pub (super) fn sys_exec(path:*const u8,context:Context)->SyscallResult{

    let name=unsafe{from_cstr(path)};
    let app = ROOT_INODE.find(name);
    match app{
        Ok(inode)=>{
            let data = inode.readall().unwrap();
            let elf = ElfFile::new(data.as_slice()).unwrap();
            let process = Process::from_elf(&elf, true).unwrap();
            let thread=Thread::new(process, elf.header.pt2.entry_point() as usize, None).unwrap();
            PROCESSOR.lock().add_thread(thread);
            PROCESSOR.lock().sleep_current_thread();
            PROCESSOR.lock().park_current_thread(&context);
            PROCESSOR.lock().prepare_next_thread();
        },
        Err(_)=>{
            println!("");
            println!("command not found");
        }
    }
    SyscallResult::Proceed(0)

}
pub unsafe fn from_cstr(s:*const u8)->&'static str{
    use core::{slice,str};
    let len=(0usize..).find(|&i| *s.add(i)==0).unwrap();
    str::from_utf8(slice::from_raw_parts(s,len)).unwrap()
}
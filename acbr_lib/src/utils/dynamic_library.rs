use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ptr;
use std::sync::Mutex;
use std::{
    env,
    ffi::CString,
    os::raw::{c_char, c_void},
};

#[cfg(target_os = "linux")]
use std::ffi::CStr;

pub unsafe fn load_library(name: *const c_char) -> *mut c_void {
    #[cfg(target_os = "windows")]
    {
        LoadLibraryA(name)
    }

    #[cfg(target_os = "linux")]
    {
        use std::ffi::CStr;
        use std::path::Path;

        let lib_name = CStr::from_ptr(name);

        let lib_path = Path::new(lib_name.to_str().unwrap_or(""));
        if !lib_path.exists() {
            eprintln!(
                "Erro: A biblioteca não foi encontrada no caminho: {:?}",
                lib_path
            );
            return std::ptr::null_mut();
        }

        let handle = dlopen(name, RTLD_NOW);
        if handle.is_null() {
            let error = CStr::from_ptr(dlerror()).to_string_lossy();
            eprintln!("Erro ao carregar biblioteca no Linux: {}", error);
        }
        handle
    }
}

// pub fn free_library(handle: *mut c_void) {
//     unsafe {
//         #[cfg(target_os = "windows")]
//         {
//             FreeLibrary(handle);
//         }

//         #[cfg(target_os = "linux")]
//         {
//             dlclose(handle);
//         }
//     }
// }

#[derive(Debug, Clone, Copy)]
struct SafeHandle(*mut c_void);

unsafe impl Send for SafeHandle {}
unsafe impl Sync for SafeHandle {}

lazy_static! {
    static ref DLL_HANDLES: Mutex<HashMap<ACBrLibType, SafeHandle>> = {
        let lib_handle: HashMap<ACBrLibType, SafeHandle> = HashMap::new();

        let mutex_lib_handle = Mutex::new(lib_handle);

        mutex_lib_handle
    };
    static ref IS_READING: Mutex<bool> = Mutex::new(false);
}

#[derive(Eq, Hash, PartialEq)]
pub enum ACBrLibType {
    Esocial,
}

pub fn unread_lib_file(lib: ACBrLibType) -> Result<(), String> {
    println!("Relendo a DLL");
    let mut handles = match DLL_HANDLES.lock() {
        Ok(handles) => handles,
        Err(_) => return Err("Não foi possivel obter o estado de leitura da DLL".into()),
    };

    if let Some(handle) = handles.get(&lib) {
        let _ = drop(handle);
        handles.remove(&lib);
    }

    Ok(())
}

pub fn read_lib_file(lib: ACBrLibType) -> Result<*mut c_void, String> {
    let mut handles = DLL_HANDLES.lock().unwrap();
    let mut is_reading = IS_READING.lock().unwrap();

    // Verifica se já está carregado
    if let Some(handle) = handles.get(&lib) {
        return Ok(handle.0);
    }

    if *is_reading {
        return Err("O arquivo da biblioteca esta sendo utilizado nesse momento!".into());
    }

    *is_reading = true;

    let mut resources_path = env::current_exe().unwrap();
    resources_path.pop();
    resources_path.push("resources");

    let mut deps_path = resources_path.clone();
    deps_path.push("deps");

    let deps_path_string = deps_path.to_str().expect("Erro ao converter para string");

    let current_path = env::var("PATH").unwrap_or_default();
    if !current_path.split(';').any(|p| p == deps_path_string) {
        env::set_var("PATH", format!("{};{}", deps_path_string, current_path));
    }

    let lib_filename = match lib {
        ACBrLibType::Esocial => "ACBreSocial32.dll",
    };

    resources_path.push(lib_filename);

    let lib_name = CString::new(resources_path.to_str().unwrap()).unwrap();

    unsafe {
        let lib_handle = load_library(lib_name.as_ptr());
        if lib_handle.is_null() {
            return Err("Erro ao carregar a DLL.".into());
        }

        // Armazena o handle no HashMap global
        let safe_handle = SafeHandle(lib_handle);
        handles.insert(lib, safe_handle);

        *is_reading = false;

        Ok(lib_handle)
    }
}

pub fn get_function(dll_handle: *mut c_void, func_name: &str) -> Result<*mut c_void, String> {
    let func_name_c = CString::new(func_name).map_err(|e| e.to_string())?;

    unsafe {
        let func = get_function_address(dll_handle, func_name_c.as_ptr());
        if func.is_null() {
            Err(format!("Erro ao localizar a função: {}", func_name))
        } else {
            Ok(func)
        }
    }
}

#[cfg(target_os = "windows")]
extern "system" {
    fn GetProcAddress(handle: *mut c_void, name: *const c_char) -> *mut c_void;
}

#[cfg(target_os = "windows")]
extern "system" {
    fn LoadLibraryA(name: *const c_char) -> *mut c_void;
    // fn FreeLibrary(handle: *mut c_void) -> i32;
}

#[cfg(target_os = "linux")]
extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlopen(filename: *const c_char, flag: i32) -> *mut c_void;
    fn dlerror() -> *const c_char;
    fn dlclose(handle: *mut c_void) -> i32;
}

#[cfg(target_os = "linux")]
const RTLD_LAZY: i32 = 0x00001;

#[cfg(target_os = "linux")]
const RTLD_NOW: i32 = 0x00002;

/// Função que faz o fallback para pegar o endereço da função dependendo do sistema operacional
unsafe fn get_function_address(handle: *mut c_void, name: *const c_char) -> *mut c_void {
    #[cfg(target_os = "windows")]
    {
        GetProcAddress(handle, name)
    }

    #[cfg(target_os = "linux")]
    {
        dlsym(handle, name)
    }
}

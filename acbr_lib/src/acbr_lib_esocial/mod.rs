use std::ffi::{CStr, CString};
// use std::fmt::Debug;
use std::mem::transmute;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
// use std::sync::OnceLock;

use crate::utils::dynamic_library::{get_function, read_lib_file, unread_lib_file, ACBrLibType};

// #[derive(Debug)]
// struct SafeHandle(*mut c_void);

// unsafe impl Send for SafeHandle {}
// unsafe impl Sync for SafeHandle {}

// static DLL_HANDLE: OnceLock<SafeHandle> = OnceLock::new();

#[derive(Clone)]
pub struct ACBrLibEsocial {
    lib_handle: *mut c_void,
    pub pointer: *mut c_void,
    config_path: Option<String>,
}

impl ACBrLibEsocial {
    pub fn new() -> Result<Self, String> {
        let lib_handle = read_lib_file(ACBrLibType::Esocial)?;

        Ok(ACBrLibEsocial {
            lib_handle,
            pointer: ptr::null_mut(),
            config_path: None,
        })
    }

    pub fn esocial_inicializar<T: AsRef<str>, U: AsRef<str>>(
        &mut self,
        arquivo_config: T,
        chave_criptografia: U,
    ) -> Result<i32, String> {
        if self.lib_handle.is_null() {
            return Err("Handle da biblioteca é nulo".to_string());
        }

        match get_function(self.lib_handle, "eSocial_Inicializar") {
            Ok(function) => unsafe {
                let esocial_inicializar: extern "stdcall" fn(
                    *mut *mut c_void,
                    *const c_char,
                    *const c_char,
                ) -> i32 = transmute(function);

                let config_path = match CString::new(arquivo_config.as_ref()) {
                    Ok(path) => path,
                    Err(_) => {
                        return Err("Falha ao converter arquivo_config".to_string());
                    }
                };

                let chave_crypt = match CString::new(chave_criptografia.as_ref()) {
                    Ok(chave) => chave,
                    Err(_) => {
                        return Err("Falha ao converter chave_criptografia".to_string());
                    }
                };

                let mut temp_pointer: *mut c_void = ptr::null_mut();

                let resultado = esocial_inicializar(
                    &mut temp_pointer,
                    config_path.as_ptr(),
                    chave_crypt.as_ptr(),
                );

                match resultado {
                    0 => {
                        if !temp_pointer.is_null() {
                            self.pointer = temp_pointer;
                            self.config_path = Some(arquivo_config.as_ref().to_string());
                            Ok(resultado)
                        } else {
                            unread_lib_file(ACBrLibType::Esocial)?;
                            self.lib_handle = ptr::null_mut();
                            Err(
                                "Ponteiro retornado é nulo mesmo com inicialização bem sucedida"
                                    .to_string(),
                            )
                        }
                    }
                    _ => {
                        unread_lib_file(ACBrLibType::Esocial)?;
                        self.lib_handle = ptr::null_mut();
                        Err(format!("Erro na inicialização: {}", resultado))
                    }
                }
            },
            Err(error) => Err(error),
        }
    }

    pub fn esocial_finalizar(&mut self) -> Result<i32, String> {
        if self.lib_handle.is_null() {
            return Err("Handle da biblioteca é nulo".to_string());
        }

        if self.pointer.is_null() {
            return Err("O ponteiro da Lib esta nulo".into());
        }

        match get_function(self.lib_handle, "eSocial_Finalizar") {
            Ok(function) => unsafe {
                let esocial_finalizar: extern "stdcall" fn(*mut c_void) -> i32 =
                    transmute(function);

                let resultado = esocial_finalizar(self.pointer);

                Ok(resultado)
            },
            Err(error) => Err(error),
        }
    }

    pub fn esocial_ultimo_retorno(&self) -> Result<String, String> {
        if self.lib_handle.is_null() {
            return Err("Handle da biblioteca é nulo".to_string());
        }

        match get_function(self.lib_handle, "eSocial_UltimoRetorno") {
            Ok(function) => unsafe {
                let esocial_ultimo_retorno: extern "stdcall" fn(
                    *mut c_void,
                    *mut c_char,
                    *mut c_int,
                ) -> c_int = transmute(function);

                let cap: usize = 1024 * 1024;
                let mut buf = vec![0u8; cap];
                let mut tamanho: c_int = cap as c_int;

                let _rc = esocial_ultimo_retorno(
                    self.pointer,
                    buf.as_mut_ptr() as *mut c_char,
                    &mut tamanho,
                );

                let message = CStr::from_ptr(buf.as_ptr() as *const c_char)
                    .to_string_lossy()
                    .into_owned();

                Ok(message)
            },
            Err(error) => Err(error),
        }
    }

    pub fn esocial_nome(&self) -> Result<String, String> {
        match get_function(self.lib_handle, "eSocial_Nome") {
            Ok(function) => unsafe {
                let esocial_nome: extern "stdcall" fn(
                    *mut c_void,
                    *mut c_char,
                    *mut c_int,
                ) -> c_int = transmute(function);

                let mut tamanho = 0;
                let resultado = esocial_nome(self.pointer, ptr::null_mut(), &mut tamanho);

                if resultado != 0 {
                    return Err(format!(
                        "Erro ao obter o tamanho necessário, código: {}",
                        resultado
                    ));
                }

                let mut buffer: Vec<u8> = vec![0; tamanho as usize + 1];

                let resultado = esocial_nome(
                    self.pointer,
                    buffer.as_mut_ptr() as *mut c_char,
                    &mut tamanho,
                );

                if resultado == 0 {
                    buffer.resize(tamanho as usize, 0);
                    if !buffer.ends_with(&[0]) {
                        buffer.push(0);
                    }

                    let message = CStr::from_bytes_with_nul(&buffer)
                        .expect("Erro ao converter a mensagem para CStr")
                        .to_string_lossy()
                        .into_owned();

                    Ok(message)
                } else {
                    Err(format!("Erro ao obter o nome, código: {}", resultado))
                }
            },
            Err(error) => Err(error),
        }
    }
    pub fn esocial_versao(&self) -> Result<String, String> {
        match get_function(self.lib_handle, "eSocial_Versao") {
            Ok(function) => unsafe {
                let esocial_versao: extern "stdcall" fn(
                    *mut c_void,
                    *mut c_char,
                    *mut c_int,
                ) -> c_int = transmute(function);

                let mut tamanho = 0;
                let resultado = esocial_versao(self.pointer, ptr::null_mut(), &mut tamanho);

                if resultado != 0 {
                    return Err(format!(
                        "Erro ao obter o tamanho necessário, código: {}",
                        resultado
                    ));
                }

                let mut buffer: Vec<u8> = vec![0; tamanho as usize + 1];

                let resultado = esocial_versao(
                    self.pointer,
                    buffer.as_mut_ptr() as *mut c_char,
                    &mut tamanho,
                );

                if resultado == 0 {
                    buffer.resize(tamanho as usize, 0);
                    if !buffer.ends_with(&[0]) {
                        buffer.push(0);
                    }

                    let message = CStr::from_bytes_with_nul(&buffer)
                        .expect("Erro ao converter a mensagem para CStr")
                        .to_string_lossy()
                        .into_owned();

                    Ok(message)
                } else {
                    Err(format!("Erro ao obter a versão, código: {}", resultado))
                }
            },
            Err(error) => Err(error),
        }
    }

    pub fn criar_evento_esocial<T: AsRef<str>>(&self, arquivo_ini: T) -> Result<i32, String> {
        match get_function(self.lib_handle, "eSocial_CriarEventoeSocial") {
            Ok(function) => unsafe {
                let criar_evento: extern "stdcall" fn(*mut c_void, *const c_char) -> i32 =
                    transmute(function);

                let arquivo_ini_path = CString::new(arquivo_ini.as_ref())
                    .expect("A conversão do parâmetro 'arquivo_ini' falhou");

                let resultado = criar_evento(self.pointer, arquivo_ini_path.as_ptr());

                Ok(resultado)
            },
            Err(error) => Err(error),
        }
    }

    pub fn enviar_esocial(&self, grupo: i32) -> Result<i32, String> {
        if self.lib_handle.is_null() {
            return Err("Handle da biblioteca é nulo".to_string());
        }

        match get_function(self.lib_handle, "eSocial_EnviareSocial") {
            Ok(function) => unsafe {
                type ESocialEnviar =
                    extern "stdcall" fn(*mut c_void, c_int, *mut c_char, *mut c_int) -> c_int;

                let enviar_esocial: ESocialEnviar = transmute(function);

                let cap: usize = 64 * 1024;
                let mut buf = vec![0u8; cap];
                let mut tamanho: c_int = cap as c_int;

                let rc = enviar_esocial(
                    self.pointer,
                    grupo as c_int,
                    buf.as_mut_ptr() as *mut c_char,
                    &mut tamanho,
                );

                Ok(rc as i32)
            },
            Err(error) => Err(error),
        }
    }

    pub fn obter_ultimo_retorno(&self) -> Result<String, String> {
        if self.lib_handle.is_null() {
            return Err("Handle da biblioteca é nulo".to_string());
        }

        match get_function(self.lib_handle, "eSocial_UltimoRetorno") {
            Ok(function) => unsafe {
                let obter_ultimo_retorno: extern "stdcall" fn(
                    *mut c_void,
                    *mut c_char,
                    *mut c_int,
                ) -> c_int = transmute(function);

                let cap: usize = 1024 * 1024;
                let mut buf = vec![0u8; cap];
                let mut tamanho: c_int = cap as c_int;

                let rc = obter_ultimo_retorno(
                    self.pointer,
                    buf.as_mut_ptr() as *mut c_char,
                    &mut tamanho,
                );

                let message = CStr::from_ptr(buf.as_ptr() as *const c_char)
                    .to_string_lossy()
                    .into_owned();

                if rc == 0 {
                    Ok(message)
                } else {
                    let message = if message.len() <= 0 {
                        "Ultimo retorno vazio".to_owned()
                    } else {
                        message
                    };
                    Err(format!("Último retorno, mensagem: {}", message))
                }
            },
            Err(error) => Err(error),
        }
    }

    pub fn consultar_protocolo<T: AsRef<str>>(&self, protocolo: T) -> Result<i32, String> {
        if self.lib_handle.is_null() {
            return Err("Handle da biblioteca é nulo".to_string());
        }

        match get_function(self.lib_handle, "eSocial_ConsultareSocial") {
            Ok(function) => unsafe {
                type ESocialConsultar = extern "stdcall" fn(
                    *mut c_void,
                    *const c_char,
                    *mut c_char,
                    *mut c_int,
                ) -> c_int;

                let consultar_protocolo: ESocialConsultar = transmute(function);

                let protocolo = CString::new(protocolo.as_ref())
                    .map_err(|_| "CString protocolo".to_string())?;

                let cap: usize = 64 * 1024;
                let mut buf = vec![0u8; cap];
                let mut tamanho: c_int = cap as c_int;

                let rc = consultar_protocolo(
                    self.pointer,
                    protocolo.as_ptr(),
                    buf.as_mut_ptr() as *mut c_char,
                    &mut tamanho,
                );

                Ok(rc as i32)
            },
            Err(error) => Err(error),
        }
    }

    pub fn criar_enviar_esocial<T: AsRef<str>>(
        &self,
        arquivo_ini: T,
        grupo: i32,
    ) -> Result<i32, String> {
        match get_function(self.lib_handle, "eSocial_CriarEnviareSocial") {
            Ok(function) => unsafe {
                let criar_enviar_esocial: extern "stdcall" fn(
                    *mut c_void,
                    *const c_char,
                    c_int,
                ) -> i32 = transmute(function);

                let arquivo_ini_path = CString::new(arquivo_ini.as_ref())
                    .expect("A conversão do parâmetro 'arquivo_ini' falhou");
                let resultado =
                    criar_enviar_esocial(self.pointer, arquivo_ini_path.as_ptr(), grupo);

                Ok(resultado)
            },
            Err(error) => Err(error),
        }
    }

    pub fn limpar_esocial(&self) -> Result<i32, String> {
        match get_function(self.lib_handle, "eSocial_LimpareSocial") {
            Ok(function) => unsafe {
                let limpar_esocial: extern "stdcall" fn(*mut c_void) -> i32 = transmute(function);

                let resultado = limpar_esocial(self.pointer);

                Ok(resultado)
            },
            Err(error) => Err(error),
        }
    }

    pub fn carregar_xml_evento<T: AsRef<str>>(&self, arquivo_ou_xml: T) -> Result<i32, String> {
        if self.lib_handle.is_null() {
            return Err("Handle da biblioteca é nulo".to_string());
        }

        match get_function(self.lib_handle, "eSocial_CarregarXMLEventoeSocial") {
            Ok(function) => unsafe {
                let carregar_xml_evento: extern "stdcall" fn(*mut c_void, *const c_char) -> i32 =
                    transmute(function);

                let xml = CString::new(arquivo_ou_xml.as_ref())
                    .expect("A conversão do parâmetro 'arquivo_ini' falhou");

                let resultado = carregar_xml_evento(self.pointer, xml.as_ptr());

                Ok(resultado)
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_id_empregador<T: AsRef<str>>(&self, id_empregador: T) -> Result<i32, String> {
        match get_function(self.lib_handle, "eSocial_SetIDEmpregador") {
            Ok(function) => unsafe {
                let set_id_empregador: extern "stdcall" fn(*mut c_void, *const c_char) -> i32 =
                    transmute(function);

                let id_empregador = CString::new(id_empregador.as_ref())
                    .expect("A conversão do parâmetro 'arquivo_ini' falhou");

                let resultado = set_id_empregador(self.pointer, id_empregador.as_ptr());

                Ok(resultado)
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_id_transmissor<T: AsRef<str>>(&self, id_transmissor: T) -> Result<i32, String> {
        match get_function(self.lib_handle, "eSocial_SetIDTransmissor") {
            Ok(function) => unsafe {
                let set_id_transmissor: extern "stdcall" fn(*mut c_void, *const c_char) -> i32 =
                    transmute(function);

                let id_transmissor = CString::new(id_transmissor.as_ref())
                    .expect("A conversão do parâmetro 'arquivo_ini' falhou");

                let resultado = set_id_transmissor(self.pointer, id_transmissor.as_ptr());

                Ok(resultado)
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_tipo_empregador(&self, tipo_empregador: i32) -> Result<i32, String> {
        match get_function(self.lib_handle, "eSocial_SetTipoEmpregador") {
            Ok(function) => unsafe {
                let set_tipo_empregador: extern "stdcall" fn(*mut c_void, c_int) -> c_int =
                    transmute(function);

                let resultado = set_tipo_empregador(self.pointer, tipo_empregador);

                Ok(resultado)
            },
            Err(error) => Err(error),
        }
    }

    pub fn set_versao_df<T: AsRef<str>>(&self, versao: T) -> Result<i32, String> {
        match get_function(self.lib_handle, "eSocial_SetVersaoDF") {
            Ok(function) => unsafe {
                let set_versao_df: extern "stdcall" fn(*mut c_void, *const c_char) -> i32 =
                    transmute(function);

                let versao = CString::new(versao.as_ref())
                    .expect("A conversão do parâmetro 'versao' falhou");

                let resultado = set_versao_df(self.pointer, versao.as_ptr());

                Ok(resultado)
            },
            Err(error) => Err(error),
        }
    }

    pub fn consulta_identificadores_eventos_empregador<T: AsRef<str>>(
        &self,
        id_empregador: T,
        tipo_evento: i32,
        periodo_apuracao: T,
    ) -> Result<i32, String> {
        match get_function(
            self.lib_handle,
            "eSocial_ConsultaIdentificadoresEventosEmpregador",
        ) {
            Ok(function) => {
                unsafe {
                    let consulta_identificadores_eventos_empregador: extern "stdcall" fn(
                        *mut c_void,
                        *const c_char, // idEmpregador
                        c_int,         // aTipoEvento
                        *const c_char, // aPeriodoApuracao
                        *mut c_char,   // sResposta
                        *mut c_int,    // esTamanho
                    )
                        -> c_int = transmute(function);

                    let id_empregador_cstr = CString::new(id_empregador.as_ref())
                        .expect("Erro ao converter id_empregador para CString");
                    let periodo_apuracao_cstr = CString::new(periodo_apuracao.as_ref())
                        .expect("Erro ao converter periodo_apuracao para CString");

                    let mut tamanho: c_int = 0;
                    let resultado = consulta_identificadores_eventos_empregador(
                        self.pointer,
                        id_empregador_cstr.as_ptr(),
                        tipo_evento,
                        periodo_apuracao_cstr.as_ptr(),
                        ptr::null_mut(),
                        &mut tamanho,
                    );

                    return Ok(resultado);
                }
            }
            Err(error) => Err(error),
        }
    }

    pub fn consulta_identificadores_eventos_tabela<T: AsRef<str>>(
        &self,
        id_empregador: T,
        tipo_evento: i32,
        chave: T,
        data_inicial: T,
        data_final: T,
    ) -> Result<i32, String> {
        match get_function(
            self.lib_handle,
            "eSocial_ConsultaIdentificadoresEventosTabela",
        ) {
            Ok(function) => {
                unsafe {
                    let consulta_identificadores_eventos_tabela: extern "stdcall" fn(
                        *mut c_void,
                        *const c_char, // idEmpregador
                        c_int,         // aTipoEvento
                        *const c_char, // aChave
                        *const c_char, // aDataInicial
                        *const c_char, // aDataFinal
                        *mut c_char,   // sResposta
                        *mut c_int,    // esTamanho
                    )
                        -> c_int = transmute(function);

                    let id_empregador_cstr = CString::new(id_empregador.as_ref())
                        .expect("Erro ao converter id_empregador para CString");
                    let chave_cstr =
                        CString::new(chave.as_ref()).expect("Erro ao converter chave para CString");
                    let data_inicial_cstr = CString::new(data_inicial.as_ref())
                        .expect("Erro ao converter data_inicial para CString");
                    let data_final_cstr = CString::new(data_final.as_ref())
                        .expect("Erro ao converter data_final para CString");

                    let mut tamanho: c_int = 0;
                    let resultado = consulta_identificadores_eventos_tabela(
                        self.pointer,
                        id_empregador_cstr.as_ptr(),
                        tipo_evento,
                        chave_cstr.as_ptr(),
                        data_inicial_cstr.as_ptr(),
                        data_final_cstr.as_ptr(),
                        ptr::null_mut(),
                        &mut tamanho,
                    );

                    return Ok(resultado);
                }
            }
            Err(error) => Err(error),
        }
    }

    pub fn consulta_identificadores_eventos_trabalhador<T: AsRef<str>>(
        &self,
        id_empregador: T,
        cpf_trabalhador: T,
        data_inicial: T,
        data_final: T,
    ) -> Result<i32, String> {
        match get_function(
            self.lib_handle,
            "eSocial_ConsultaIdentificadoresEventosTrabalhador",
        ) {
            Ok(function) => {
                unsafe {
                    let consulta_identificadores_eventos_trabalhador: extern "stdcall" fn(
                        *mut c_void,
                        *const c_char, // idEmpregador
                        *const c_char, // aCPFTrabalhador
                        *const c_char, // aDataInicial
                        *const c_char, // aDataFinal
                        *mut c_char,   // sResposta
                        *mut c_int,    // esTamanho
                    )
                        -> c_int = transmute(function);

                    let id_empregador_cstr = CString::new(id_empregador.as_ref())
                        .expect("Erro ao converter id_empregador para CString");
                    let cpf_trabalhador_cstr = CString::new(cpf_trabalhador.as_ref())
                        .expect("Erro ao converter cpf_trabalhador para CString");
                    let data_inicial_cstr = CString::new(data_inicial.as_ref())
                        .expect("Erro ao converter data_inicial para CString");
                    let data_final_cstr = CString::new(data_final.as_ref())
                        .expect("Erro ao converter data_final para CString");

                    let mut tamanho: c_int = 0;
                    let resultado = consulta_identificadores_eventos_trabalhador(
                        self.pointer,
                        id_empregador_cstr.as_ptr(),
                        cpf_trabalhador_cstr.as_ptr(),
                        data_inicial_cstr.as_ptr(),
                        data_final_cstr.as_ptr(),
                        ptr::null_mut(),
                        &mut tamanho,
                    );

                    return Ok(resultado);
                }
            }
            Err(error) => Err(error),
        }
    }

    pub fn download_eventos<T: AsRef<str>>(
        &self,
        id_empregador: T,
        cpf_trabalhador: T,
        data_inicial: T,
        data_final: T,
    ) -> Result<i32, String> {
        match get_function(self.lib_handle, "eSocial_DownloadEventos") {
            Ok(function) => {
                unsafe {
                    let download_eventos: extern "stdcall" fn(
                        *mut c_void,
                        *const c_char, // idEmpregador
                        *const c_char, // aCPFTrabalhador
                        *const c_char, // aDataInicial
                        *const c_char, // aDataFinal
                        *mut c_char,   // sResposta
                        *mut c_int,    // esTamanho
                    ) -> c_int = transmute(function);

                    let id_empregador_cstr = CString::new(id_empregador.as_ref())
                        .expect("Erro ao converter id_empregador para CString");
                    let cpf_trabalhador_cstr = CString::new(cpf_trabalhador.as_ref())
                        .expect("Erro ao converter cpf_trabalhador para CString");
                    let data_inicial_cstr = CString::new(data_inicial.as_ref())
                        .expect("Erro ao converter data_inicial para CString");
                    let data_final_cstr = CString::new(data_final.as_ref())
                        .expect("Erro ao converter data_final para CString");

                    let mut tamanho: c_int = 0;
                    let resultado = download_eventos(
                        self.pointer,
                        id_empregador_cstr.as_ptr(),
                        cpf_trabalhador_cstr.as_ptr(),
                        data_inicial_cstr.as_ptr(),
                        data_final_cstr.as_ptr(),
                        ptr::null_mut(),
                        &mut tamanho,
                    );

                    return Ok(resultado);
                }
            }
            Err(error) => Err(error),
        }
    }

    pub fn obter_certificados(&self) -> Result<i32, String> {
        match get_function(self.lib_handle, "eSocial_ObterCertificados") {
            Ok(function) => {
                unsafe {
                    let obter_certificados: extern "stdcall" fn(
                        *mut c_void,
                        *mut c_char, // sResposta
                        *mut c_int,  // esTamanho
                    ) -> c_int = transmute(function);

                    let mut tamanho: c_int = 0;
                    let resultado = obter_certificados(self.pointer, ptr::null_mut(), &mut tamanho);

                    return Ok(resultado);
                }
            }
            Err(error) => Err(error),
        }
    }

    pub fn validar_esocial(&self) -> Result<i32, String> {
        match get_function(self.lib_handle, "eSocial_Validar") {
            Ok(function) => unsafe {
                let validar_esocial: extern "stdcall" fn(*mut c_void) -> i32 = transmute(function);

                let resultado = validar_esocial(self.pointer);

                Ok(resultado)
            },
            Err(error) => Err(error),
        }
    }
    pub fn esocial_config_ler<T: AsRef<str>>(&self, arquivo_config: T) -> Result<i32, String> {
        match get_function(self.lib_handle, "eSocial_ConfigLer") {
            Ok(function) => unsafe {
                let esocial_config_ler: extern "stdcall" fn(*mut c_void, *const c_char) -> i32 =
                    transmute(function);

                let config_path = CString::new(arquivo_config.as_ref())
                    .expect("A conversão do parâmetro 'arquivo_config' falhou");

                let resultado = esocial_config_ler(self.pointer, config_path.as_ptr());

                Ok(resultado)
            },
            Err(error) => Err(error),
        }
    }

    pub fn esocial_config_gravar<T: AsRef<str>>(&self, arquivo_config: T) -> Result<i32, String> {
        match get_function(self.lib_handle, "eSocial_ConfigGravar") {
            Ok(function) => unsafe {
                let esocial_config_gravar: extern "stdcall" fn(*mut c_void, *const c_char) -> i32 =
                    transmute(function);

                let config_path = CString::new(arquivo_config.as_ref())
                    .expect("A conversão do parâmetro 'arquivo_config' falhou");

                let resultado = esocial_config_gravar(self.pointer, config_path.as_ptr());

                Ok(resultado)
            },
            Err(error) => Err(error),
        }
    }

    pub fn esocial_config_ler_valor<T: AsRef<str>>(
        &self,
        sessao: T,
        chave: T,
    ) -> Result<String, String> {
        match get_function(self.lib_handle, "eSocial_ConfigLerValor") {
            Ok(function) => unsafe {
                let esocial_config_ler_valor: extern "stdcall" fn(
                    *mut c_void,
                    *const c_char,
                    *const c_char,
                    *mut c_char,
                    *mut c_int,
                ) -> i32 = transmute(function);

                let sessao = CString::new(sessao.as_ref())
                    .expect("A conversão do parâmetro 'sessao' falhou");

                let chave =
                    CString::new(chave.as_ref()).expect("A conversão do parâmetro 'chave' falhou");

                let mut tamanho = 0;

                let _resultado = esocial_config_ler_valor(
                    self.pointer,
                    sessao.as_ptr(),
                    chave.as_ptr(),
                    ptr::null_mut(),
                    &mut tamanho,
                );

                let mut buffer: Vec<u8> = vec![0; tamanho as usize + 1];

                let _resultado = esocial_config_ler_valor(
                    self.pointer,
                    sessao.as_ptr(),
                    chave.as_ptr(),
                    buffer.as_mut_ptr() as *mut c_char,
                    &mut tamanho,
                );

                buffer.resize(tamanho as usize, 0);

                if !buffer.ends_with(&[0]) {
                    buffer.push(0);
                }

                let message = CStr::from_bytes_with_nul(&buffer)
                    .expect("Erro ao converter a mensagem para CStr")
                    .to_string_lossy()
                    .into_owned();

                Ok(message)
            },
            Err(error) => Err(error),
        }
    }

    pub fn esocial_config_gravar_valor<T: AsRef<str>>(
        &self,
        sessao: T,
        chave: T,
        valor: T,
    ) -> Result<i32, String> {
        match get_function(self.lib_handle, "eSocial_ConfigGravarValor") {
            Ok(function) => unsafe {
                let esocial_config_gravar_valor: extern "stdcall" fn(
                    *mut c_void,
                    *const c_char,
                    *const c_char,
                    *const c_char,
                ) -> i32 = transmute(function);

                let sessao = CString::new(sessao.as_ref())
                    .expect("A conversão do parâmetro 'arquivo_config' falhou");

                let chave = CString::new(chave.as_ref())
                    .expect("A conversão do parâmetro 'arquivo_config' falhou");

                let valor = CString::new(valor.as_ref())
                    .expect("A conversão do parâmetro 'arquivo_config' falhou");

                let resultado = esocial_config_gravar_valor(
                    self.pointer,
                    sessao.as_ptr(),
                    chave.as_ptr(),
                    valor.as_ptr(),
                );

                Ok(resultado)
            },
            Err(error) => Err(error),
        }
    }

    pub fn esocial_config_importar<T: AsRef<str>>(&self, arquivo_config: T) -> Result<i32, String> {
        match get_function(self.lib_handle, "eSocial_ConfigImportar") {
            Ok(function) => unsafe {
                let esocial_config_importar: extern "stdcall" fn(
                    *mut c_void,
                    *const c_char,
                ) -> i32 = transmute(function);

                let config_path = CString::new(arquivo_config.as_ref())
                    .expect("A conversão do parâmetro 'arquivo_config' falhou");

                let resultado = esocial_config_importar(self.pointer, config_path.as_ptr());

                Ok(resultado)
            },
            Err(error) => Err(error),
        }
    }

    pub fn esocial_config_exportar(&self) -> Result<String, String> {
        match get_function(self.lib_handle, "eSocial_ConfigExportar") {
            Ok(function) => unsafe {
                let esocial_config_exportar: extern "stdcall" fn(
                    *mut c_void,
                    *mut c_char,
                    *mut c_int,
                ) -> i32 = transmute(function);

                let mut tamanho = 0;

                let resultado =
                    esocial_config_exportar(self.pointer, ptr::null_mut(), &mut tamanho);

                if resultado != 0 {
                    return Err(format!(
                        "Erro ao obter o tamanho necessário, código: {}",
                        resultado
                    ));
                }

                let mut buffer: Vec<u8> = vec![0; tamanho as usize + 1];

                let resultado = esocial_config_exportar(
                    self.pointer,
                    buffer.as_mut_ptr() as *mut c_char,
                    &mut tamanho,
                );

                if resultado == 0 {
                    buffer.resize(tamanho as usize, 0);

                    if !buffer.ends_with(&[0]) {
                        buffer.push(0);
                    }

                    let message = CStr::from_bytes_with_nul(&buffer)
                        .expect("Erro ao converter a mensagem para CStr")
                        .to_string_lossy()
                        .into_owned();

                    Ok(message)
                } else {
                    Err(format!(
                        "Erro ao obter o último retorno, código: {}",
                        resultado
                    ))
                }
            },
            Err(error) => Err(error),
        }
    }
}

unsafe impl Send for ACBrLibEsocial {}
unsafe impl Sync for ACBrLibEsocial {}

impl Drop for ACBrLibEsocial {
    fn drop(&mut self) {
        if let Some(config_path) = &self.config_path {
            if std::path::Path::new(config_path).exists() {
                let _ = std::fs::remove_file(config_path);
            }
        }

        let _ = self.esocial_finalizar();
    }
}
